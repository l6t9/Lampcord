package me.lampu.lampcord.shared.gateway

import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.client.request.header
import io.ktor.websocket.CloseReason
import io.ktor.websocket.Frame
import io.ktor.websocket.close
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
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.add
import kotlinx.serialization.json.boolean
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put
import me.lampu.lampcord.shared.api.AuthApi
import me.lampu.lampcord.shared.api.RestClient
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.model.GatewayPayload
import me.lampu.lampcord.shared.model.Identify
import me.lampu.lampcord.shared.model.IdentifyClientState
import me.lampu.lampcord.shared.model.Resume
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.getCpuCoreCount
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import me.lampu.lampcord.shared.utils.getDeviceName
import me.lampu.lampcord.shared.utils.getMemoryMemory
import me.lampu.lampcord.shared.utils.getOsSdkVersion
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.randomUUID
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds

class GatewayManager(
    private val client: HttpClient,
    private val authApi: AuthApi,
    private val rest: RestClient,
    private val readStateStore: me.lampu.lampcord.shared.state.ReadStateStore? = null,
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

    private val guildSubscriptions = mutableMapOf<String, GuildSubscriptionState>()

    private class GuildSubscriptionState {
        var typing: Boolean = true
        var threads: Boolean = true
        var activities: Boolean = true
        val members = mutableSetOf<String>()
        val channels = mutableMapOf<String, List<List<Int>>>()
        val threadMemberLists = mutableSetOf<String>()
        private val channelOrder = mutableListOf<String>()

        fun updateChannel(channelId: String, listId: String, ranges: List<List<Int>>, isThread: Boolean) {
            if (isThread) {
                threadMemberLists.add(channelId)
            } else {
                channels[channelId] = ranges
                // We use channelId for LRU tracking and map key
                channelOrder.remove(channelId)
                channelOrder.add(channelId)
                if (channelOrder.size > 5) {
                    val oldest = channelOrder.removeAt(0)
                    channels.remove(oldest)
                }
            }
        }
    }

    fun connect(token: String) {
        disconnect()
        Logging.i("Gateway", "Connecting to Discord Gateway...")
        connectionJob = scope.launch {
            try {
                val gatewayUrl = authApi.getGatewayUrl() ?: "wss://gateway.discord.gg"
                val url = (resumeGatewayUrl ?: gatewayUrl).removeSuffix("/") + "/?v=9&encoding=json"
                Logging.d("Gateway", "Gateway URL: $url")

                val userAgent = when (val platform = getPlatformName()) {
                    "android" -> {
                        "Discord-Android/341200;RNA"
                    }
                    "ios" -> {
                        "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
                    }
                    else -> {
                        val browserOsName = when (platform) {
                            "windows" -> "Windows NT 10.0; Win64; x64"
                            "linux" -> "X11; Linux x86_64"
                            "macos" -> "Macintosh; Intel Mac OS X 10_15_7"
                            else -> "X11; Linux x86_64"
                        }
                        "Mozilla/5.0 ($browserOsName) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
                    }
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
                    Logging.i("Gateway", "WebSocket connected.")
                    session = this
                    reconnectAttempt = 0
                    
                    while (isActive) {
                        try {
                            val frame = incoming.receive()
                            if (frame is Frame.Text) {
                                val text = frame.readText()
                                val payload = json.decodeFromString<GatewayPayload>(text)
                                lastSequence = payload.s ?: lastSequence
                                handlePayload(payload, token)
                                _events.emit(payload)
                            } else if (frame is Frame.Close) {
                                val reason = closeReason.await()
                                Logging.w("Gateway", "WebSocket closed: $reason")
                                val code = reason?.code?.toInt() ?: 0
                                if (code == 4004 || code == 4003) {
                                    _events.emit(GatewayPayload(op = -1, t = "AUTH_FAILED"))
                                    disconnect()
                                    return@webSocket
                                }
                            }
                        } catch (e: Exception) {
                            Logging.e("Gateway", "Error in WebSocket loop", e)
                            break
                        }
                    }
                }
            } catch (e: Exception) {
                Logging.e("Gateway", "Connection failed", e)
            } finally {
                Logging.i("Gateway", "Cleaning up connection...")
                session = null
                stopHeartbeat()
                stopTimeSpentUpdates()
                stopHelloTimeout()
                
                if (isActive) {
                    reconnectAttempt++
                    val delay = (1000 * (1 shl (reconnectAttempt - 1))).milliseconds.coerceAtMost(maxReconnectDelay)
                    Logging.i("Gateway", "Reconnecting in $delay (attempt $reconnectAttempt)...")
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
            10 -> { 
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
            0 -> { 
                when (payload.t) {
                    "READY" -> {
                        val data = payload.d?.jsonObject
                        sessionId = data?.get("session_id")?.jsonPrimitive?.content
                        resumeGatewayUrl = data?.get("resume_gateway_url")?.jsonPrimitive?.content
                        Logging.i("Gateway", "READY. Session: $sessionId")
                        startTimeSpentUpdates()
                    }
                    "RESUMED" -> {
                        Logging.i("Gateway", "RESUMED session $sessionId")
                    }
                }
            }
            1 -> { 
                sendHeartbeat()
            }
            7 -> { 
                disconnect()
                connect(token)
            }
            9 -> { 
                val resumable = payload.d?.jsonPrimitive?.boolean ?: false
                Logging.w("Gateway", "Invalid session. Resumable: $resumable")
                if (!resumable) {
                    sessionId = null
                    lastSequence = null
                }
                delay(1000.milliseconds)
                identify(token)
            }
            11 -> {
                heartbeatAckReceived = true
            }
        }
    }

    private fun startHeartbeat(interval: Long) {
        Logging.d("Gateway", "Starting heartbeat every ${interval}ms")
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
        Logging.i("Gateway", "Resuming session $sessionId at sequence $lastSequence")
        val resume = Resume(
            token = token,
            session_id = sessionId!!,
            seq = lastSequence!!
        )
        val payload = GatewayPayload(op = 6, d = json.encodeToJsonElement(resume))
        sendPayload(payload)
    }

    private suspend fun identify(token: String) {
        Logging.i("Gateway", "Identifying...")
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"
        
        val properties = buildMap {
            if (isMobile) {
                put("os", JsonPrimitive(if (platform == "android") "Android" else "iOS"))
                put("browser", JsonPrimitive(if (platform == "android") "Discord Android" else "Discord iOS"))
                put("device", JsonPrimitive(getDeviceName()))
            } else {
                put("os", JsonPrimitive(platform.replaceFirstChar { it.uppercase() }))
                put("browser", JsonPrimitive("Discord Desktop"))
                put("device", JsonPrimitive(""))
            }
            put("system_locale", JsonPrimitive("en-US"))
            put("has_client_mods", JsonPrimitive(false))
            put("client_version", JsonPrimitive(if (isMobile) "341.0 - rn" else "0.0.309"))
            put("release_channel", JsonPrimitive("stable"))
            put("device_vendor_id", JsonPrimitive(rest.vendorId))
            put("design_id", JsonPrimitive(if (isMobile) 2 else 0))
            put("browser_user_agent", JsonPrimitive(""))
            put("browser_version", JsonPrimitive(""))
            put("os_version", JsonPrimitive(getOsSdkVersion()))
            put("client_build_number", JsonPrimitive(if (isMobile) 6081 else 309000))
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

        val highestLastMessageId = readStateStore?.readStates?.value?.values
            ?.mapNotNull { it.lastMessageId() }
            ?.maxByOrNull { it.toLongOrNull() ?: 0L }
            ?.toLongOrNull() ?: 0L

        val identify = Identify(
            token = token,
            properties = properties,
            capabilities = 351,
            large_threshold = 100,
            compress = false, 
            client_state = IdentifyClientState(
                guild_hashes = emptyMap(),
                highest_last_message_id = highestLastMessageId,
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

    fun updatePresence(status: String?, activities: List<Activity>?) {
        val payload = GatewayPayload(
            op = 3,
            d = buildJsonObject {
                put("status", status?.let { JsonPrimitive(it) } ?: JsonNull)
                put("activities", buildJsonArray {
                    activities?.forEach { activity ->
                        add(json.encodeToJsonElement<Activity>(activity))
                    }
                })
                put("afk", JsonPrimitive(false))
                put("since", JsonPrimitive(0))
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendSubscription(guildId: String) {
        val state = guildSubscriptions.getOrPut(guildId) { GuildSubscriptionState() }
        val payload = GatewayPayload(
            op = 14,
            d = buildJsonObject {
                put("guild_id", guildId.toLong())
                put("typing", state.typing)
                put("threads", state.threads)
                put("activities", state.activities)
                put("members", buildJsonArray { state.members.forEach { add(it.toLong()) } })
                put("channels", buildJsonObject { 
                    state.channels.forEach { (chanId, ranges) ->
                        put(chanId, buildJsonArray {
                            ranges.forEach { range ->
                                add(buildJsonArray { add(range[0]); add(range[1]) })
                            }
                        })
                    }
                })
                put("thread_member_lists", buildJsonArray { state.threadMemberLists.forEach { add(it.toLong()) } })
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendVoiceStateUpdate(guildId: String?, channelId: String?, selfMute: Boolean = false, selfDeaf: Boolean = false, selfVideo: Boolean = false) {
        val payload = GatewayPayload(
            op = 4,
            d = buildJsonObject {
                put("guild_id", guildId?.toLongOrNull())
                put("channel_id", channelId?.toLongOrNull())
                put("self_mute", JsonPrimitive(selfMute))
                put("self_deaf", JsonPrimitive(selfDeaf))
                put("self_video", JsonPrimitive(selfVideo))
            }
        )
        scope.launch { sendPayload(payload) }
    }

    fun sendLazyRequest(guildId: String, channelId: String, listId: String, ranges: List<List<Int>>, isThread: Boolean = false) {
        val state = guildSubscriptions.getOrPut(guildId) { GuildSubscriptionState() }
        
        state.updateChannel(channelId, listId, ranges, isThread)

        val payload = GatewayPayload(
            op = 14,
            d = buildJsonObject {
                put("guild_id", guildId.toLong())
                put("typing", state.typing)
                put("threads", state.threads)
                put("activities", state.activities)
                put("members", buildJsonArray { state.members.forEach { add(it.toLong()) } })
                put("thread_member_lists", buildJsonArray { state.threadMemberLists.forEach { add(it.toLong()) } })
                put("channels", buildJsonObject {
                    state.channels.forEach { (lid, r) ->
                        put(lid, buildJsonArray {
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
