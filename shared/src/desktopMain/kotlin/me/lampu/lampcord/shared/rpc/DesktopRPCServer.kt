package me.lampu.lampcord.shared.rpc

import io.ktor.server.application.*
import io.ktor.server.engine.EmbeddedServer
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.routing.*
import io.ktor.server.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.utils.Logging
import kotlin.time.Duration.Companion.seconds

class DesktopRPCServer(private val gatewayManager: GatewayManager) {
    private var server: EmbeddedServer<*, *>? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val json = Json { ignoreUnknownKeys = true }

    fun start() {
        if (server != null) return

        scope.launch {
            for (port in 6463..6472) {
                try {
                    Logging.i("RPC", "Attempting to start RPC server on port $port")
                    val engine = embeddedServer(Netty, port = port) {
                        install(WebSockets) {
                            pingPeriod = 15.seconds
                            timeout = 15.seconds
                            maxFrameSize = Long.MAX_VALUE
                            masking = false
                        }
                        routing {
                            webSocket("/") {
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

    fun stop() {
        server?.stop(1000, 2000)
        server = null
    }
}
