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
import kotlin.time.Duration.Companion.seconds

import me.lampu.lampcord.shared.api.DiscordClient

class GatewayManager(
    private val client: HttpClient,
    private val discordClient: DiscordClient,
    private val json: Json = Json { 
        ignoreUnknownKeys = true 
        explicitNulls = false
    }
) {
    private var session: DefaultClientWebSocketSession? = null
    private var connectionJob: Job? = null
    private val _events = MutableSharedFlow<GatewayPayload>()
    val events: SharedFlow<GatewayPayload> = _events.asSharedFlow()

    private var heartbeatJob: Job? = null
    private var heartbeatAckReceived = true
    private var timeSpentJob: Job? = null
    private var helloTimeoutJob: Job? = null
    private var lastSequence: Int? = null
    var sessionId: String? = null
    private var resumeGatewayUrl: String? = null
    private var clientHeartbeatSessionId = randomUUID()
    private val clientLaunchId = randomUUID()
    private val launchSignature = (getCurrentTimeMillis() * 1_000_000L).toString()
    
    private val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())

    private var reconnectAttempt = 0
    private val maxReconnectDelay = 30.seconds

    // Subscription tracking (Stability Parity with 126.21 GuildSubscriptionsManager)
    private val guildSubscriptions = mutableMapOf<String, GuildSubscriptionState>()

    private data class GuildSubscriptionState(
        val typing: Boolean = true,
        val threads: Boolean = true,
        val activities: Boolean = true,
        val members: List<String> = emptyList(),
        val channels: MutableMap<String, List<List<Int>>> = mutableMapOf()
    )

    fun connect(token: String) {
        disconnect()
        connectionJob = scope.launch {
            try {
                val gatewayUrl = discordClient.getGatewayUrl() ?: "wss://gateway.discord.gg"
                val url = (resumeGatewayUrl ?: gatewayUrl).removeSuffix("/") + "/?v=9&encoding=json"

                val platform = getPlatformName()
                val userAgent = if (platform == "android") {
                    "Discord-Android/341200;RNA"
                } else if (platform == "ios") {
                    "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
                } else {
                    val browserOsName = when(platform) {
                        "windows" -> "Windows NT 10.0; Win64; x64"
                        "linux" -> "X11; Linux x86_64"
                        "macos" -> "Macintosh; Intel Mac OS X 10_15_7"
                        else -> "X11; Linux x86_64"
                    }
                    "Mozilla/5.0 ($browserOsName) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
                }

                startHelloTimeout(token)

                client.webSocket(
                    urlString = url,
                    request = {
                        header("User-Agent", userAgent)
                        header("Origin", "https://discord.com")
                        header("Accept-Language", "en-US,en;q=0.9")
                        header("Cache-Control", "no-cache")
                    }
                ) {
                    session = this
                    reconnectAttempt = 0
                    
                    while (isActive) {
                        val frame = incoming.receive()
                        if (frame is Frame.Text) {
                            val text = frame.readText()
                            val payload = json.decodeFromString<GatewayPayload>(text)
                            lastSequence = payload.s ?: lastSequence
                            handlePayload(payload, token)
                            _events.emit(payload)
                        }
                    }
                }
            } catch (e: Exception) {
            } finally {
                session = null
                stopHeartbeat()
                stopTimeSpentUpdates()
                stopHelloTimeout()
                
                if (isActive) {
                    reconnectAttempt++
                    val delay = (1000 * (1 shl (reconnectAttempt - 1))).milliseconds.coerceAtMost(maxReconnectDelay)
                    delay(delay)
                    connect(token)
                }
            }
        }
    }

    fun disconnect() {
        connectionJob?.cancel()
        connectionJob = null
        session = null
        stopHeartbeat()
        stopTimeSpentUpdates()
        stopHelloTimeout()
        guildSubscriptions.clear()
    }

    private suspend fun handlePayload(payload: GatewayPayload, token: String) {
        when (payload.op) {
            10 -> { // Hello
                stopHelloTimeout()
                val heartbeatInterval = payload.d?.jsonObject?.get("heartbeat_interval")?.jsonPrimitive?.let {
                    it.longOrNull ?: it.doubleOrNull?.toLong()
                } ?: 41250
                startHeartbeat(heartbeatInterval)
                
                if (sessionId != null && lastSequence != null) {
                    resume(token)
                } else {
                    identify(token)
                }
            }
            0 -> { // Dispatch
                when (payload.t) {
                    "READY" -> {
                        val data = payload.d?.jsonObject
                        sessionId = data?.get("session_id")?.jsonPrimitive?.content
                        resumeGatewayUrl = data?.get("resume_gateway_url")?.jsonPrimitive?.content
                        startTimeSpentUpdates()
                    }
                }
            }
            1 -> { // Heartbeat requested
                sendHeartbeat()
            }
            7 -> { // Reconnect
                disconnect()
                connect(token)
            }
            9 -> { // Invalid Session
                val resumable = payload.d?.jsonPrimitive?.boolean ?: false
                if (!resumable) {
                    sessionId = null
                    lastSequence = null
                }
                delay(1000)
                identify(token)
            }
            11 -> { // Heartbeat ACK
                heartbeatAckReceived = true
            }
        }
    }

    private fun startHeartbeat(interval: Long) {
        heartbeatJob?.cancel()
        heartbeatAckReceived = true
        heartbeatJob = CoroutineScope(Dispatchers.Default).launch {
            while (isActive) {
                delay(interval.milliseconds)
                if (!heartbeatAckReceived) {
                    session?.close(CloseReason(CloseReason.Codes.VIOLATED_POLICY, "Heartbeat ACK timeout"))
                    return@launch
                }
                heartbeatAckReceived = false
                sendHeartbeat()
            }
        }
    }

    private fun stopHeartbeat() {
        heartbeatJob?.cancel()
        heartbeatJob = null
    }

    private fun startHelloTimeout(token: String) {
        helloTimeoutJob?.cancel()
        helloTimeoutJob = scope.launch {
            delay(20.seconds)
            disconnect()
            connect(token)
        }
    }

    private fun stopHelloTimeout() {
        helloTimeoutJob?.cancel()
        helloTimeoutJob = null
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
            op = 1,
            d = lastSequence?.let { JsonPrimitive(it) } ?: JsonNull
        )
        sendPayload(payload)
    }

    fun sendExpeditedHeartbeat() {
        scope.launch {
            sendHeartbeat()
        }
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

    private suspend fun resume(token: String) {
        val resume = Resume(
            token = token,
            session_id = sessionId!!,
            seq = lastSequence!!
        )
        val payload = GatewayPayload(op = 6, d = json.encodeToJsonElement(resume))
        sendPayload(payload)
    }

    private suspend fun identify(token: String) {
        val properties = buildMap {
            put("os", JsonPrimitive("Android"))
            put("browser", JsonPrimitive("Discord Android"))
            put("device", JsonPrimitive(getDeviceName()))
            put("system_locale", JsonPrimitive("en-US"))
            put("has_client_mods", JsonPrimitive(false))
            put("client_version", JsonPrimitive("341.0 - rn"))
            put("release_channel", JsonPrimitive("canaryRelease"))
            put("device_vendor_id", JsonPrimitive(discordClient.vendorId))
            put("design_id", JsonPrimitive(2))
            put("browser_user_agent", JsonPrimitive(""))
            put("browser_version", JsonPrimitive(""))
            put("os_version", JsonPrimitive(getOsSdkVersion()))
            put("client_build_number", JsonPrimitive(6081))
            put("client_event_source", JsonNull)
            put("client_launch_id", JsonPrimitive(clientLaunchId))
            put("launch_signature", JsonPrimitive(launchSignature))
            put("client_heartbeat_session_id", JsonPrimitive(clientHeartbeatSessionId))
            put("client_app_state", JsonPrimitive("active"))
            put("accessibility_features", JsonPrimitive(0))
            put("accessibility_support_enabled", JsonPrimitive(false))
            put("client_performance_cpu", JsonPrimitive(getCpuCoreCount()))
            put("client_performance_memory", JsonPrimitive(getMemoryMemory()))
            put("cpu_core_count", JsonPrimitive(getCpuCoreCount()))
        }

        val identify = Identify(
            token = token,
            properties = properties,
            capabilities = 351, 
            large_threshold = 100,
            compress = false, 
            client_state = IdentifyClientState(
                guild_hashes = emptyMap(),
                highest_last_message_id = 0,
                read_state_version = 0,
                user_guild_settings_version = -1
            )
        )
        val payload = GatewayPayload(op = 2, d = json.encodeToJsonElement(identify))
        sendPayload(payload)
    }

    suspend fun sendPayload(payload: GatewayPayload) {
        val jsonString = json.encodeToString(payload)
        session?.send(jsonString)
    }

    fun sendSubscription(guildId: String) {
        val state = guildSubscriptions.getOrPut(guildId) { GuildSubscriptionState() }
        val payload = GatewayPayload(
            op = 14,
            d = buildJsonObject {
                put("guild_id", guildId)
                put("typing", state.typing)
                put("threads", state.threads)
                put("activities", state.activities)
                put("members", buildJsonArray { state.members.forEach { add(it) } })
                put("channels", buildJsonObject { 
                    state.channels.forEach { (chanId, ranges) ->
                        put(chanId, buildJsonArray {
                            ranges.forEach { range ->
                                add(buildJsonArray { add(range[0]); add(range[1]) })
                            }
                        })
                    }
                })
                put("thread_member_lists", buildJsonArray { })
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendVoiceStateUpdate(guildId: String?, channelId: String?, selfMute: Boolean = false, selfDeaf: Boolean = false, selfVideo: Boolean = false) {
        val payload = GatewayPayload(
            op = 4,
            d = buildJsonObject {
                put("guild_id", guildId?.let { JsonPrimitive(it) } ?: JsonNull)
                put("channel_id", channelId?.let { JsonPrimitive(it) } ?: JsonNull)
                put("self_mute", JsonPrimitive(selfMute))
                put("self_deaf", JsonPrimitive(selfDeaf))
                put("self_video", JsonPrimitive(selfVideo))
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendLazyRequest(guildId: String, channelId: String, ranges: List<List<Int>>) {
        val state = guildSubscriptions.getOrPut(guildId) { GuildSubscriptionState() }
        
        state.channels.clear() 
        state.channels[channelId] = ranges

        val payload = GatewayPayload(
            op = 14,
            d = buildJsonObject {
                put("guild_id", guildId)
                put("typing", state.typing)
                put("threads", state.threads)
                put("activities", state.activities)
                put("members", buildJsonArray { state.members.forEach { add(it) } })
                put("thread_member_lists", buildJsonArray { })
                put("channels", buildJsonObject {
                    state.channels.forEach { (cid, r) ->
                        put(cid, buildJsonArray {
                            r.forEach { range ->
                                add(buildJsonArray {
                                    add(range[0])
                                    add(range[1])
                                })
                            }
                        })
                    }
                })
            }
        )
        scope.launch { sendPayload(payload) }
    }
}
