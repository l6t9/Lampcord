package me.lampu.lampcord.shared.gateway

import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.randomUUID
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import io.ktor.client.*
import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import kotlinx.serialization.json.*
import kotlinx.serialization.encodeToString
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes

class GatewayManager(
    private val client: HttpClient,
    private val json: Json = Json { ignoreUnknownKeys = true }
) {
    private var session: DefaultClientWebSocketSession? = null
    private var connectionJob: Job? = null
    private val _events = MutableSharedFlow<GatewayPayload>()
    val events: SharedFlow<GatewayPayload> = _events.asSharedFlow()

    private var heartbeatJob: Job? = null
    private var timeSpentJob: Job? = null
    private var lastSequence: Int? = null
    private var sessionId: String? = null
    private var clientHeartbeatSessionId = randomUUID()

    fun connect(token: String) {
        disconnect()
        connectionJob = CoroutineScope(Dispatchers.Default).launch {
            try {
                // Gateway v9: both Paicord and d.py-self connect with v9 for user accounts;
                // Discord's v10 gateway silently disables features (incl. member list subscriptions)
                // for user accounts that don't declare capabilities in IDENTIFY.
                client.webSocket("wss://gateway.discord.gg/?v=9&encoding=json") {
                    session = this
                    
                    // Start listening for messages
                    while (isActive) {
                        val frame = incoming.receive()
                        if (frame is Frame.Text) {
                            val payload = json.decodeFromString<GatewayPayload>(frame.readText())
                            lastSequence = payload.s ?: lastSequence
                            handlePayload(payload, token)
                            _events.emit(payload)
                        }
                    }
                }
            } catch (e: Exception) {
                println("Gateway Error: ${e.message}")
            } finally {
                session = null
                stopHeartbeat()
                stopTimeSpentUpdates()
            }
        }
    }

    fun disconnect() {
        connectionJob?.cancel()
        connectionJob = null
        session = null
        stopHeartbeat()
        stopTimeSpentUpdates()
    }

    private suspend fun handlePayload(payload: GatewayPayload, token: String) {
        when (payload.op) {
            10 -> { // Hello
                val heartbeatInterval = payload.d?.jsonObject?.get("heartbeat_interval")?.jsonPrimitive?.long ?: 41250
                startHeartbeat(heartbeatInterval)
                identify(token)
            }
            0 -> { // Dispatch
                if (payload.t == "READY") {
                    sessionId = payload.d?.jsonObject?.get("session_id")?.jsonPrimitive?.content
                    startTimeSpentUpdates()
                }
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

    private fun startTimeSpentUpdates() {
        timeSpentJob?.cancel()
        timeSpentJob = CoroutineScope(Dispatchers.Default).launch {
            while (isActive) {
                sendUpdateTimeSpent()
                delay(30.minutes)
                // Cycle client heartbeat session ID every 30 mins like Paicord
                clientHeartbeatSessionId = randomUUID()
            }
        }
    }

    private fun stopTimeSpentUpdates() {
        timeSpentJob?.cancel()
        timeSpentJob = null
    }

    private suspend fun sendHeartbeat() {
        // Paicord uses Op 40 (QOS_HEARTBEAT) instead of Op 1 for pings
        // when established, which seems to help with member list stability.
        val payload = GatewayPayload(
            op = 40, 
            d = buildJsonObject {
                put("seq", lastSequence?.let { JsonPrimitive(it) } ?: JsonNull)
                put("qos", buildJsonObject {
                    put("ver", 1)
                    put("active", true)
                    put("reasons", buildJsonArray { add("foregrounded") })
                })
            }
        )
        sendPayload(payload)
    }

    private suspend fun sendUpdateTimeSpent() {
        val payload = GatewayPayload(
            op = 41,
            d = buildJsonObject {
                // initialization_timestamp: Unix timestamp in ms
                put("initialization_timestamp", JsonPrimitive(getCurrentTimeMillis()))
                // session_id: client_heartbeat_session_id from identify
                put("session_id", JsonPrimitive(clientHeartbeatSessionId))
                // client_launch_id: can be fixed or random per session
                put("client_launch_id", JsonPrimitive(clientHeartbeatSessionId))
            }
        )
        sendPayload(payload)
    }

    private suspend fun identify(token: String) {
        val platform = getPlatformName()
        val discordOs = when (platform) {
            "macos" -> "Mac OS X"
            "android" -> "Android"
            "ios" -> "iOS"
            "linux" -> "Linux"
            "windows" -> "Windows"
            else -> platform.replaceFirstChar { it.uppercase() }
        }
        
        val isMobile = platform == "android" || platform == "ios"
        
        val identify = Identify(
            token = token,
            properties = IdentifyProperties(
                os = discordOs,
                browser = when (discordOs) {
                    "iOS", "Android" -> "Discord ${discordOs}"
                    "Mac OS X" -> "Discord Client"
                    else -> "Safari"
                },
                device = "Lampcord ${if (isMobile) "Mobile" else "Desktop"}",
                release_channel = "stable",
                client_version = if (isMobile) "336.0" else "0.0.398",
                os_version = when (platform) {
                    "macos" -> "24.5.0"
                    "ios" -> "18.0"
                    "android" -> "15.0"
                    else -> "1.0"
                },
                os_arch = if (platform == "macos") "arm64" else "x64",
                system_locale = "en-US",
                client_build_number = when (discordOs) {
                    "iOS", "Android" -> 105180
                    "Mac OS X" -> 575562
                    else -> null
                },
                client_app_state = if (isMobile) "active" else "focused",
                client_heartbeat_session_id = clientHeartbeatSessionId
            ),
            // Byte-identical to Paicord's working user-account IDENTIFY:
            // token + properties + capabilities. No `intents` (bot-only field,
            // users receive all events regardless) and no `compress`.
            //
            // Mirrors Paicord's UserGatewayManager defaults EXCEPT userSettingsProto:
            // lazyUserNotes(0) | nonChannelReadStates(7) | authTokenRefresh(8) |
            // debounceMessageReactions(13) = 8577
            capabilities = 8577
        )
        val payload = GatewayPayload(op = 2, d = json.encodeToJsonElement(identify))
        sendPayload(payload)
    }

    suspend fun sendPayload(payload: GatewayPayload) {
        val jsonString = json.encodeToString(payload)
        session?.send(jsonString)
    }
}
