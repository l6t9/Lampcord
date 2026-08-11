package me.lampu.lampcord.shared.gateway

import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.websocket.Frame
import io.ktor.websocket.readText
import io.ktor.websocket.send
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import me.lampu.lampcord.shared.model.VoiceGatewayPayload
import me.lampu.lampcord.shared.model.VoiceIdentify
import me.lampu.lampcord.shared.model.VoiceReady
import kotlin.time.Duration.Companion.milliseconds

class VoiceGatewayManager(
    private val client: HttpClient,
    private val json: Json = Json { ignoreUnknownKeys = true }
) {
    private var session: DefaultClientWebSocketSession? = null
    private var connectionJob: Job? = null
    private var heartbeatJob: Job? = null
    
    private val _events = MutableSharedFlow<VoiceGatewayPayload>()
    val events: SharedFlow<VoiceGatewayPayload> = _events.asSharedFlow()

    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    fun connect(endpoint: String, guildId: String, userId: String, sessionId: String, token: String) {
        disconnect()
        val url = "wss://${endpoint.replace(":80", "")}/?v=8"
        println("Connecting to Voice Gateway: $url")
        
        connectionJob = scope.launch {
            try {
                client.webSocket(url) {
                    session = this
                    println("Voice Gateway Connected")
                    
                    while (isActive) {
                        val frame = incoming.receive()
                        if (frame is Frame.Text) {
                            val text = frame.readText()
                            try {
                                val payload = json.decodeFromString<VoiceGatewayPayload>(text)
                                handlePayload(payload, guildId, userId, sessionId, token)
                                _events.emit(payload)
                            } catch (e: Exception) {
                                println("Voice Gateway Parse Error: ${e.message}")
                                println("Raw JSON: $text")
                            }
                        }
                    }
                }
            } catch (e: Exception) {
                println("Voice Gateway Error: ${e.message}")
            } finally {
                session = null
                stopHeartbeat()
            }
        }
    }

    fun disconnect() {
        connectionJob?.cancel()
        connectionJob = null
        session = null
        stopHeartbeat()
    }

    private suspend fun handlePayload(payload: VoiceGatewayPayload, guildId: String, userId: String, sessionId: String, token: String) {
        when (payload.op) {
            8 -> { // Hello
                val heartbeatInterval = payload.d?.jsonObject?.get("heartbeat_interval")?.jsonPrimitive?.let {
                    it.longOrNull ?: it.doubleOrNull?.toLong()
                } ?: 41250
                startHeartbeat(heartbeatInterval)
                identify(guildId, userId, sessionId, token)
            }
            2 -> { // Ready
                val ready = payload.d?.let { json.decodeFromJsonElement<VoiceReady>(it) }
                if (ready != null) {
                    println("Voice Gateway Ready: SSRC=${ready.ssrc}, Endpoint=${ready.ip}:${ready.port}")
                    // To be fully functional, we'd perform IP discovery here
                    // and then call selectProtocol.
                }
            }
            4 -> { // Session Description
                println("Voice Gateway Session Description Received")
            }
            6 -> { // Heartbeat ACK
            }
        }
    }

    private fun startHeartbeat(interval: Long) {
        heartbeatJob?.cancel()
        heartbeatJob = CoroutineScope(Dispatchers.Default).launch {
            while (isActive) {
                delay(interval.milliseconds)
                sendHeartbeat()
            }
        }
    }

    private fun stopHeartbeat() {
        heartbeatJob?.cancel()
        heartbeatJob = null
    }

    private suspend fun sendHeartbeat() {
        val payload = VoiceGatewayPayload(op = 3, d = JsonPrimitive(kotlin.random.Random.nextInt()))
        sendPayload(payload)
    }

    private suspend fun identify(guildId: String, userId: String, sessionId: String, token: String) {
        val identify = VoiceIdentify(
            server_id = guildId,
            user_id = userId,
            session_id = sessionId,
            token = token,
            video = true
        )
        val payload = VoiceGatewayPayload(op = 0, d = json.encodeToJsonElement(identify))
        sendPayload(payload)
    }

    private suspend fun sendPayload(payload: VoiceGatewayPayload) {
        val jsonString = json.encodeToString(payload)
        session?.send(jsonString)
    }
}
