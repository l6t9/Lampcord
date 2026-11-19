package me.lampu.lampcord.shared.rpc

import io.ktor.server.application.*
import io.ktor.server.engine.EmbeddedServer
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.routing.*
import io.ktor.server.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.oauth.OAuthAuthorizationHandler
import me.lampu.lampcord.shared.oauth.OAuthRequestSource
import me.lampu.lampcord.shared.oauth.OAuthResult
import me.lampu.lampcord.shared.oauth.OAuthUrlParser
import me.lampu.lampcord.shared.utils.Logging
import java.net.URI
import java.net.URLEncoder
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds

private const val MAX_RPC_FRAME_BYTES = 1024 * 1024
private val AUTHORIZE_TIMEOUT = 5.minutes

class DesktopRPCServer(
    private val gatewayManager: GatewayManager,
    private val oauthAuthorizationHandler: OAuthAuthorizationHandler,
) {
    private var server: EmbeddedServer<*, *>? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val json = Json { ignoreUnknownKeys = true }

    fun start() {
        if (server != null) return

        scope.launch {
            for (port in 6463..6472) {
                try {
                    Logging.i("RPC", "Attempting to start RPC server on loopback port $port")
                    // Bind loopback only: this socket can change the user's Discord presence,
                    // so it must not be reachable from another machine on the network.
                    // The Origin check below also stops remote browser pages from driving it;
                    // native RPC clients send no Origin and are allowed through.
                    val engine = embeddedServer(Netty, host = "127.0.0.1", port = port) {
                        install(WebSockets) {
                            pingPeriod = 15.seconds
                            timeout = 15.seconds
                            maxFrameSize = MAX_RPC_FRAME_BYTES.toLong()
                            masking = false
                        }
                        routing {
                            webSocket("/") {
                                val origin = call.request.headers["Origin"]
                                if (origin != null && !isLocalOrigin(origin)) {
                                    Logging.w("RPC", "Rejected RPC connection with non-local Origin: $origin")
                                    close(CloseReason(CloseReason.Codes.VIOLATED_POLICY, "Non-local origin"))
                                    return@webSocket
                                }
                                handleConnection()
                            }
                        }
                    }
                    server = engine
                    engine.start(wait = false)
                    Logging.i("RPC", "RPC server started on port $port")
                    break
                } catch (e: Exception) {
                    Logging.w("RPC", "Failed to start RPC server on port $port: ${e.message}")
                }
            }
        }
    }

    private suspend fun DefaultWebSocketServerSession.handleConnection() {
        Logging.d("RPC", "New RPC connection")
        try {
            send(buildJsonObject {
                put("cmd", "DISPATCH")
                put("data", buildJsonObject {
                    put("v", 1)
                    put("config", buildJsonObject {
                        put("cdn_host", "cdn.discordapp.com")
                        put("api_endpoint", "//discord.com/api")
                        put("environment", "production")
                    })
                })
                put("evt", "READY")
                put("nonce", JsonNull)
            }.toString())

            for (frame in incoming) {
                if (frame is Frame.Text) {
                    val text = frame.readText()
                    val payload = json.parseToJsonElement(text).jsonObject
                    val cmd = payload["cmd"]?.jsonPrimitive?.content
                    val nonce = payload["nonce"]?.jsonPrimitive?.content

                    when (cmd) {
                        "SET_ACTIVITY" -> {
                            val data = payload["args"]?.jsonObject
                            val activityJson = data?.get("activity")
                            if (activityJson != null) {
                                try {
                                    val activity = json.decodeFromJsonElement<Activity>(activityJson)
                                    Logging.i("RPC", "Received activity: ${activity.name}")
                                    gatewayManager.updatePresence(null, listOf(activity))
                                } catch (e: Exception) {
                                    Logging.e("RPC", "Error decoding activity: ${e.message}")
                                }
                            }
                            
                            send(buildJsonObject {
                                put("cmd", "SET_ACTIVITY")
                                put("data", activityJson ?: JsonNull)
                                put("evt", JsonNull)
                                put("nonce", nonce)
                            }.toString())
                        }
                        "AUTHORIZE" -> handleAuthorize(payload, nonce)
                        "SUBSCRIBE" -> {
                        }
                    }
                }
            }
        } catch (e: Exception) {
            Logging.e("RPC", "RPC connection error: ${e.message}")
        } finally {
            Logging.d("RPC", "RPC connection closed")
        }
    }

    private suspend fun DefaultWebSocketServerSession.handleAuthorize(
        payload: JsonObject,
        nonce: String?,
    ) {
        val request = requestFromRpcArgs(payload["args"]?.jsonObject)
        if (request == null) {
            sendAuthorizeError(nonce, "invalid_request")
            return
        }

        val reply = launch {
            val result = withTimeoutOrNull(AUTHORIZE_TIMEOUT) {
                oauthAuthorizationHandler.results.first { it.requestId == request.id }
            }
            if (result == null) {
                sendAuthorizeError(nonce, "authorization_timeout")
                return@launch
            }
            send(buildJsonObject {
                put("cmd", "AUTHORIZE")
                put("evt", JsonNull)
                put("nonce", nonce)
                put("data", buildJsonObject {
                    when (result) {
                        is OAuthResult.Success -> {
                            put("code", result.code)
                            put("redirect_uri", result.redirectUrl)
                        }
                        is OAuthResult.Denied -> {
                            put("error", result.error)
                            result.redirectUrl?.let { put("redirect_uri", it) }
                        }
                        is OAuthResult.Failed -> {
                            put("error", result.error)
                            result.message?.let { put("error_description", it) }
                        }
                    }
                })
            }.toString())
        }

        if (!oauthAuthorizationHandler.beginAuthorization(request)) {
            reply.cancel()
            sendAuthorizeError(nonce, "expired_request")
        }
    }

    private fun requestFromRpcArgs(args: JsonObject?) = runCatching {
        if (args == null) return@runCatching null
        val params = LinkedHashMap<String, String>()
        args["client_id"]?.jsonPrimitive?.contentOrNull?.let { params["client_id"] = it }
        val scopeElement = args["scopes"] ?: args["scope"]
        val scope = when (scopeElement) {
            is JsonArray -> scopeElement.mapNotNull { it.jsonPrimitive.contentOrNull }.joinToString(" ")
            is JsonPrimitive -> scopeElement.contentOrNull
            else -> null
        }
        scope?.let { params["scope"] = it }
        params["response_type"] = args["response_type"]?.jsonPrimitive?.contentOrNull ?: "code"
        args["redirect_uri"]?.jsonPrimitive?.contentOrNull?.let { params["redirect_uri"] = it }
        args["state"]?.jsonPrimitive?.contentOrNull?.let { params["state"] = it }
        args["nonce"]?.jsonPrimitive?.contentOrNull?.let { params["nonce"] = it }
        args["code_challenge"]?.jsonPrimitive?.contentOrNull?.let { params["code_challenge"] = it }
        args["code_challenge_method"]?.jsonPrimitive?.contentOrNull?.let { params["code_challenge_method"] = it }
        args["prompt"]?.jsonPrimitive?.contentOrNull?.let { params["prompt"] = it }
        args["integration_type"]?.jsonPrimitive?.contentOrNull?.let { params["integration_type"] = it }
        val query = params.entries.joinToString("&") { (key, value) ->
            "${encodeParam(key)}=${encodeParam(value)}"
        }
        OAuthUrlParser.parse("https://discord.com/oauth2/authorize?$query", OAuthRequestSource.DESKTOP_RPC)
    }.getOrNull()

    private fun encodeParam(value: String): String = URLEncoder.encode(value, "UTF-8")

    private suspend fun DefaultWebSocketServerSession.sendAuthorizeError(nonce: String?, error: String) {
        send(buildJsonObject {
            put("cmd", "AUTHORIZE")
            put("evt", JsonNull)
            put("nonce", nonce)
            put("data", buildJsonObject { put("error", error) })
        }.toString())
    }

    private fun isLocalOrigin(origin: String): Boolean = runCatching {
        when (URI(origin).host?.lowercase()) {
            "localhost", "127.0.0.1", "::1", "[::1]" -> true
            else -> false
        }
    }.getOrDefault(false)

    fun stop() {
        server?.stop(1000, 2000)
        server = null
    }
}
