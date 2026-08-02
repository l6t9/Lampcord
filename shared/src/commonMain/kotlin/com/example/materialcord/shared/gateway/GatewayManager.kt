package com.example.materialcord.shared.gateway

import com.example.materialcord.shared.model.*
import com.example.materialcord.shared.utils.getPlatformName
import io.ktor.client.*
import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import kotlinx.serialization.json.*
import kotlinx.serialization.encodeToString
import kotlin.time.Duration.Companion.milliseconds

class GatewayManager(
    private val client: HttpClient,
    private val json: Json = Json { ignoreUnknownKeys = true }
) {
    private var session: DefaultClientWebSocketSession? = null
    private val _events = MutableSharedFlow<GatewayPayload>()
    val events: SharedFlow<GatewayPayload> = _events.asSharedFlow()

    private var heartbeatJob: Job? = null
    private var lastSequence: Int? = null

    suspend fun connect(token: String) {
        client.webSocket("wss://gateway.discord.gg/?v=10&encoding=json") {
            session = this
            
            // Start listening for messages
            val readJob = launch {
                try {
                    for (frame in incoming) {
                        if (frame is Frame.Text) {
                            val payload = json.decodeFromString<GatewayPayload>(frame.readText())
                            lastSequence = payload.s ?: lastSequence
                            handlePayload(payload, token)
                            _events.emit(payload)
                        }
                    }
                } catch (e: Exception) {
                    println("Gateway Error: ${e.message}")
                } finally {
                    stopHeartbeat()
                }
            }

            readJob.join()
        }
    }

    private suspend fun handlePayload(payload: GatewayPayload, token: String) {
        when (payload.op) {
            10 -> { // Hello
                val heartbeatInterval = payload.d?.jsonObject?.get("heartbeat_interval")?.jsonPrimitive?.long ?: 41250
                startHeartbeat(heartbeatInterval)
                identify(token)
            }
            1 -> { // Heartbeat requested
                sendHeartbeat()
            }
            11 -> { // Heartbeat ACK
                // Handle ACK if needed
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
        val payload = GatewayPayload(op = 1, d = lastSequence?.let { JsonPrimitive(it) } ?: JsonNull)
        sendPayload(payload)
    }

    private suspend fun identify(token: String) {
        val identify = Identify(
            token = token,
            properties = IdentifyProperties(
                os = getPlatformName(),
                browser = "Materialcord",
                device = "Materialcord"
            ),
            intents = 32767 // All intents for now (requires care in production)
        )
        val payload = GatewayPayload(op = 2, d = json.encodeToJsonElement(identify))
        sendPayload(payload)
    }

    private suspend fun sendPayload(payload: GatewayPayload) {
        session?.send(json.encodeToString(payload))
    }
}
