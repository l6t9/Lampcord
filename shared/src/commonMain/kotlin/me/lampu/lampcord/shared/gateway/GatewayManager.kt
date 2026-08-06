package me.lampu.lampcord.shared.gateway

import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.*
import io.ktor.client.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.request.*
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
    var sessionId: String? = null
    private var clientHeartbeatSessionId = randomUUID()
    private val clientLaunchId = randomUUID()
    private val launchSignature = "8d6a888b-20b7-46b6-9e74-6b912c1dc515" // Paicord's static UUID bitmask pattern
    
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    fun connect(token: String) {
        disconnect()
        connectionJob = scope.launch {
            try {
                val platform = getPlatformName()
                val isMobile = platform == "android" || platform == "ios"
                val userAgent = if (isMobile) {
                    if (platform == "android") {
                        "Discord Android/300.0"
                    } else {
                        "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
                    }
                } else {
                    val browserOsName = when(platform) {
                        "windows" -> "Windows NT 10.0; Win64; x64"
                        "linux" -> "X11; Linux x86_64"
                        "macos" -> "Macintosh; Intel Mac OS X 10_15_7"
                        else -> "X11; Linux x86_64"
                    }
                    "Mozilla/5.0 ($browserOsName) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
                }

                client.webSocket(
                    urlString = "wss://gateway.discord.gg/?v=9&encoding=json",
                    request = {
                        header("User-Agent", userAgent)
                        header("Origin", "https://discord.com")
                        header("Accept-Language", "en-US,en;q=0.9")
                        header("Cache-Control", "no-cache")
                    }
                ) {
                    session = this
                    
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
                val heartbeatInterval = payload.d?.jsonObject?.get("heartbeat_interval")?.jsonPrimitive?.let {
                    it.longOrNull ?: it.doubleOrNull?.toLong()
                } ?: 41250
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
                clientHeartbeatSessionId = randomUUID()
            }
        }
    }

    private fun stopTimeSpentUpdates() {
        timeSpentJob?.cancel()
        timeSpentJob = null
    }

    private suspend fun sendHeartbeat() {
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
                put("initialization_timestamp", JsonPrimitive(getCurrentTimeMillis()))
                put("session_id", JsonPrimitive(clientHeartbeatSessionId))
                put("client_launch_id", JsonPrimitive(clientHeartbeatSessionId))
            }
        )
        sendPayload(payload)
    }

    private suspend fun identify(token: String) {
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"
        
        val discordOs = when (platform) {
            "macos" -> "Mac OS X"
            "android" -> "Android"
            "ios" -> "iOS"
            "linux" -> "Linux"
            "windows" -> "Windows"
            else -> platform.replaceFirstChar { it.uppercase() }
        }

        val properties = IdentifyProperties(
            os = discordOs,
            browser = when (platform) {
                "android" -> "Discord Android"
                "ios" -> "Discord iOS"
                else -> "Discord Client"
            },
            device = getDeviceName(),
            system_locale = "en-US",
            client_version = if (isMobile) "300.0" else "0.0.398",
            os_version = getOsVersion(),
            os_arch = getOsArch(),
            app_arch = getOsArch(),
            has_client_mods = false,
            client_launch_id = clientLaunchId,
            browser_user_agent = if (isMobile) "" else {
                val browserOsName = when(platform) {
                    "windows" -> "Windows NT 10.0; Win64; x64"
                    "linux" -> "X11; Linux x86_64"
                    "macos" -> "Macintosh; Intel Mac OS X 10_15_7"
                    else -> "X11; Linux x86_64"
                }
                "Mozilla/5.0 ($browserOsName) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
            },
            browser_version = if (isMobile) "" else "37.6.0",
            os_sdk_version = if (isMobile) getOsVersion() else "24",
            client_build_number = if (isMobile) 105180 else 575562,
            native_build_number = if (isMobile) null else 85861,
            client_event_source = null,
            launch_signature = if (platform == "ios") getCurrentTimeMillis().toString() else launchSignature,
            client_heartbeat_session_id = clientHeartbeatSessionId,
            client_app_state = if (isMobile) "active" else "focused",
            release_channel = "stable"
        )

        val identify = Identify(
            token = token,
            properties = properties,
            capabilities = 8577, // Reverted to 8577 to disable userSettingsProto and restore JSON user_settings
            intents = null,
            large_threshold = 50,
            compress = null
        )
        val payload = GatewayPayload(op = 2, d = json.encodeToJsonElement(identify))
        sendPayload(payload)
    }

    suspend fun sendPayload(payload: GatewayPayload) {
        val jsonString = json.encodeToString(payload)
        session?.send(jsonString)
    }

    fun sendSubscription(guildId: String) {
        val payload = GatewayPayload(
            op = 14,
            d = buildJsonObject {
                put("guild_id", guildId)
                put("typing", true)
                put("threads", true)
                put("activities", true)
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendVoiceStateUpdate(guildId: String?, channelId: String?, selfMute: Boolean = false, selfDeaf: Boolean = false, selfVideo: Boolean = false, flags: Int = 0) {
        val payload = GatewayPayload(
            op = 4,
            d = buildJsonObject {
                put("guild_id", guildId?.let { JsonPrimitive(it) } ?: JsonNull)
                put("channel_id", channelId?.let { JsonPrimitive(it) } ?: JsonNull)
                put("self_mute", JsonPrimitive(selfMute))
                put("self_deaf", JsonPrimitive(selfDeaf))
                put("self_video", JsonPrimitive(selfVideo))
                put("flags", JsonPrimitive(flags))
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendLazyRequest(guildId: String, channelId: String, ranges: List<List<Int>>) {
        val payload = GatewayPayload(
            op = 14,
            d = buildJsonObject {
                put("guild_id", guildId)
                put("typing", true)
                put("threads", true)
                put("activities", true)
                put("channels", buildJsonObject {
                    put(channelId, buildJsonArray {
                        ranges.forEach { range ->
                            add(buildJsonArray {
                                add(range[0])
                                add(range[1])
                            })
                        }
                    })
                })
            }
        )
        scope.launch { sendPayload(payload) }
    }
}
