package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.SnapshotStateMap
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.collectLatest
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.gateway.*
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.voice.notifyIncomingCall
import me.lampu.lampcord.shared.voice.startVoiceSession
import me.lampu.lampcord.shared.voice.stopVoiceSession

class VoiceStore(
    private val gatewayManager: GatewayManager,
    private val voiceGatewayManager: VoiceGatewayManager,
    private val channelApi: ChannelApi,
    private val userStore: UserStore,
    private val guildStore: GuildStore,
    private val settingsStore: SettingsStore,
    private val json: Json,
    private val scope: CoroutineScope
) : GatewayEventHandler {
    override val supportedEvents = setOf("VOICE_STATE_UPDATE", "VOICE_SERVER_UPDATE", "CALL_CREATE", "CALL_UPDATE", "CALL_DELETE", "READY_SUPPLEMENTAL", "GUILD_CREATE", "GUILD_DELETE",
        "STREAM_CREATE", "STREAM_SERVER_UPDATE", "STREAM_UPDATE", "STREAM_DELETE")
    var currentVoiceState by mutableStateOf<VoiceState?>(null)
        private set
    var activeChannel by mutableStateOf<Channel?>(null)
        private set
    var connection by mutableStateOf(VoiceConnectionStatus())
        private set
    val isVoiceConnected get() = connection.phase == VoicePhase.CONNECTED || connection.phase == VoicePhase.SECURE
    var voiceConnectionDuration by mutableStateOf(0L)
        private set
    var error by mutableStateOf<String?>(null)
    var selfMuted by mutableStateOf(false)
        private set
    var selfDeafened by mutableStateOf(false)
        private set
    var speakerEnabled by mutableStateOf(false)
        private set
    var isVoiceChatTextVisible by mutableStateOf(false)
    val voiceStates = mutableStateMapOf<String, SnapshotStateMap<String, VoiceState>>()
    val calls = mutableStateMapOf<String, List<String>>()
    var incomingChannelId by mutableStateOf<String?>(null)
        private set
    /** Latest camera frame per participant of the joined call. */
    val videoFrames get() = voiceGatewayManager.videoFrames
    val streamFrame get() = voiceGatewayManager.streamFrame
    /** Key of the Go Live stream being watched, see [streamKey]. */
    var watchedStream by mutableStateOf<String?>(null)
        private set
    var streamConnection by mutableStateOf(VoiceConnectionStatus())
        private set
    var streamPaused by mutableStateOf(false)
        private set
    private var streamServerId: String? = null
    private var streamServer: Pair<String, String>? = null
    private var connectedStream: Pair<String, String>? = null
    private var server: VoiceServerUpdate? = null
    private var connectedServer: VoiceServerUpdate? = null
    private var joinTimeout: Job? = null
    private var timer: Job? = null
    private var ringJob: Job? = null
    private var outgoing = false

    init {
        scope.launch {
            snapshotFlow { incomingChannelId }.collectLatest { channel ->
                runCatching { notifyIncomingCall(channel.takeUnless { settingsStore.userSettings?.status == "dnd" }) }
                if (channel != null) { delay(30_000); incomingChannelId = null }
            }
        }
        scope.launch {
            voiceGatewayManager.status.collect { status ->
                if (status.channelId != activeChannel?.id) return@collect
                connection = status
                if (status.phase == VoicePhase.FAILED) {
                    fail(status.error ?: "Voice connection failed")
                } else if (isVoiceConnected) {
                    joinTimeout?.cancel()
                    if (timer == null) timer = scope.launch {
                        val started = kotlin.time.TimeSource.Monotonic.markNow()
                        while (isActive) { voiceConnectionDuration = started.elapsedNow().inWholeSeconds; delay(1000) }
                    }
                }
            }
        }
        scope.launch {
            voiceGatewayManager.streamStatus.collect { status ->
                if (status.channelId != watchedStream) return@collect
                streamConnection = status
                if (status.phase == VoicePhase.FAILED) {
                    stopWatchingStream()
                    error = status.error ?: "Could not watch the stream."
                }
            }
        }
    }

    fun streamKey(channel: Channel, userId: String) =
        if (channel.guild_id != null) "guild:${channel.guild_id}:${channel.id}:$userId" else "call:${channel.id}:$userId"

    /** Starts watching [userId]'s Go Live stream in the joined call. */
    fun watchStream(userId: String) {
        val channel = activeChannel ?: return
        val key = streamKey(channel, userId)
        if (watchedStream == key) return
        stopWatchingStream()
        watchedStream = key
        streamConnection = VoiceConnectionStatus(key, VoicePhase.CONNECTING)
        scope.launch { gatewayManager.sendPayload(GatewayPayload(op = 20, d = buildJsonObject { put("stream_key", key) })) }
    }

    fun stopWatchingStream() {
        val key = watchedStream ?: return
        stopStreamLocal()
        scope.launch { gatewayManager.sendPayload(GatewayPayload(op = 19, d = buildJsonObject { put("stream_key", key) })) }
    }

    private fun stopStreamLocal() {
        watchedStream = null
        streamServerId = null
        streamServer = null
        connectedStream = null
        streamPaused = false
        streamConnection = VoiceConnectionStatus()
        voiceGatewayManager.stopStream()
    }

    private fun connectStream() {
        val key = watchedStream ?: return
        val rtcServerId = streamServerId ?: return
        val (endpoint, token) = streamServer ?: return
        val state = currentVoiceState ?: return
        if (state.session_id.isEmpty() || connectedStream == streamServer) return
        connectedStream = streamServer
        voiceGatewayManager.watchStream(endpoint, rtcServerId, key, state.user_id, state.session_id, token)
    }

    fun connectToVoice(channel: Channel, ring: Boolean = false) {
        if (channel.type !in listOf(1, 2, 3)) {
            error = "Only server voice channels and DM calls support DAVE here. Stage channels are not supported."
            return
        }
        if (userStore.currentUser.value == null || channel.id.toULongOrNull() == null) return
        if (activeChannel?.id == channel.id) return
        disconnectFromVoice()
        error = null
        try { startVoiceSession() } catch (_: Exception) {
            error = "Microphone access is unavailable. Allow microphone permission and start the call while the app is open."
            return
        }
        activeChannel = channel
        currentVoiceState = null
        server = null
        connectedServer = null
        outgoing = ring && channel.guild_id == null
        awaitConnection(channel)
        voiceGatewayManager.setMuted(selfMuted, selfDeafened)
        voiceGatewayManager.setSpeaker(speakerEnabled)
        gatewayManager.sendVoiceStateUpdate(channel.guild_id, channel.id, selfMuted, selfDeafened)
        if (incomingChannelId == channel.id) {
            incomingChannelId = null
            scope.launch { channelApi.stopRinging(channel.id) }
        }
    }

    private fun awaitConnection(channel: Channel) {
        connection = VoiceConnectionStatus(channel.id, VoicePhase.CONNECTING)
        joinTimeout?.cancel()
        joinTimeout = scope.launch {
            delay(45_000)
            if (activeChannel?.id == channel.id && !isVoiceConnected) fail("Joining voice timed out. Check your channel permissions and network.")
        }
    }

    fun disconnectFromVoice(fallbackGuildId: String? = null) {
        val channel = activeChannel
        if (channel != null) {
            gatewayManager.sendVoiceStateUpdate(channel.guild_id ?: fallbackGuildId, null, true, true)
            if (outgoing && channel.guild_id == null) scope.launch { channelApi.stopRinging(channel.id, all = true) }
        }
        stopLocal()
    }

    private fun stopLocal() {
        stopStreamLocal()
        activeChannel?.let { channel -> userStore.currentUser.value?.id?.let { voiceStates[channel.guild_id ?: "@me"]?.remove(it) } }
        activeChannel = null
        currentVoiceState = null
        server = null
        connectedServer = null
        outgoing = false
        joinTimeout?.cancel(); joinTimeout = null
        ringJob?.cancel(); ringJob = null
        timer?.cancel(); timer = null
        voiceConnectionDuration = 0
        voiceGatewayManager.disconnect()
        stopVoiceSession()
        connection = VoiceConnectionStatus()
    }

    private fun fail(message: String) { disconnectFromVoice(); error = message }

    fun toggleVoiceMute() {
        selfMuted = !selfMuted
        updateMute()
    }

    fun toggleVoiceDeaf() {
        selfDeafened = !selfDeafened
        updateMute()
    }

    private fun updateMute() {
        voiceGatewayManager.setMuted(selfMuted || currentVoiceState?.mute == true || currentVoiceState?.suppress == true,
            selfDeafened || currentVoiceState?.deaf == true)
        currentVoiceState = currentVoiceState?.copy(self_mute = selfMuted, self_deaf = selfDeafened)
        activeChannel?.let { gatewayManager.sendVoiceStateUpdate(it.guild_id, it.id, selfMuted, selfDeafened) }
    }

    fun toggleSpeaker() {
        speakerEnabled = !speakerEnabled
        voiceGatewayManager.setSpeaker(speakerEnabled)
    }

    fun declineCall(channelId: String) {
        if (incomingChannelId == channelId) incomingChannelId = null
        scope.launch { if (!channelApi.stopRinging(channelId)) error = "Could not decline the call. Please retry." }
    }

    fun requestCall(channel: Channel) {
        if (channel.type == 1 || channel.type == 3) scope.launch {
            gatewayManager.sendPayload(GatewayPayload(op = 13, d = buildJsonObject { put("channel_id", channel.id) }))
        }
    }

    override fun handleEvent(type: String, data: JsonElement?) {
        val value = data as? JsonObject ?: return
        when (type) {
            "VOICE_STATE_UPDATE" -> stateUpdate(json.decodeFromJsonElement(value))
            "VOICE_SERVER_UPDATE" -> {
                val update = json.decodeFromJsonElement<VoiceServerUpdate>(value)
                val channel = activeChannel ?: return
                if (update.guild_id != channel.guild_id || (update.channel_id != null && update.channel_id != channel.id)) return
                server = update
                if (update.endpoint == null) {
                    connectedServer = null
                    voiceGatewayManager.disconnect()
                    awaitConnection(channel)
                } else connectGateway()
            }
            "CALL_CREATE", "CALL_UPDATE" -> {
                val id = value.getValue("channel_id").jsonPrimitive.content
                val ringing = value["ringing"]?.jsonArray?.map { it.jsonPrimitive.content } ?: calls[id].orEmpty()
                calls[id] = ringing
                value["voice_states"]?.jsonArray?.forEach { stateUpdate(json.decodeFromJsonElement(it), snapshot = true) }
                val self = userStore.currentUser.value?.id
                if (self in ringing && activeChannel?.id != id) incomingChannelId = id
                else if (incomingChannelId == id) incomingChannelId = null
            }
            "CALL_DELETE" -> {
                val id = value.getValue("channel_id").jsonPrimitive.content
                calls.remove(id)
                if (incomingChannelId == id) incomingChannelId = null
                voiceStates["@me"]?.entries?.filter { it.value.channel_id == id }?.forEach { voiceStates["@me"]?.remove(it.key) }
                if (activeChannel?.id == id) stopLocal()
            }
            "STREAM_CREATE", "STREAM_UPDATE" -> {
                if (value["stream_key"]?.jsonPrimitive?.content != watchedStream) return
                streamPaused = value["paused"]?.jsonPrimitive?.booleanOrNull == true
                value["rtc_server_id"]?.jsonPrimitive?.contentOrNull?.let { streamServerId = it }
                connectStream()
            }
            "STREAM_SERVER_UPDATE" -> {
                if (value["stream_key"]?.jsonPrimitive?.content != watchedStream) return
                val endpoint = value["endpoint"]?.jsonPrimitive?.contentOrNull ?: return
                streamServer = endpoint to value.getValue("token").jsonPrimitive.content
                connectStream()
            }
            "STREAM_DELETE" -> {
                if (value["stream_key"]?.jsonPrimitive?.content != watchedStream) return
                stopStreamLocal()
                val reason = value["reason"]?.jsonPrimitive?.contentOrNull
                if (reason != null && reason != "user_requested" && reason != "stream_ended") error = "The stream is no longer available ($reason)."
            }
            "READY_SUPPLEMENTAL" -> seedGuilds(value)
            "GUILD_CREATE" -> seedGuild(value)
            "GUILD_DELETE" -> {
                val id = value.getValue("id").jsonPrimitive.content
                voiceStates.remove(id)
                if (activeChannel?.guild_id == id) disconnectFromVoice()
            }
        }
    }

    fun handleReady(data: JsonObject) {
        stopLocal()
        voiceStates.clear(); calls.clear(); incomingChannelId = null
        seedGuilds(data)
    }

    private fun seedGuilds(data: JsonObject) {
        data["guilds"]?.jsonArray?.forEach { seedGuild(it.jsonObject) }
    }

    private fun seedGuild(guild: JsonObject) {
        val id = guild["id"]?.jsonPrimitive?.content ?: return
        guild["voice_states"]?.jsonArray?.forEach { raw ->
            val state = json.decodeFromJsonElement<VoiceState>(raw).copy(guild_id = id)
            stateUpdate(state, snapshot = true)
        }
    }

    private fun stateUpdate(state: VoiceState, snapshot: Boolean = false) {
        val map = voiceStates.getOrPut(state.guild_id ?: "@me") { mutableStateMapOf() }
        if (state.channel_id == null) map.remove(state.user_id) else map[state.user_id] = state
        state.member?.let { member -> state.guild_id?.let { userStore.cacheMember(it, state.user_id, member) } }
        if (snapshot || state.user_id != userStore.currentUser.value?.id) return
        val desired = activeChannel ?: return
        val previous = currentVoiceState
        if (state.channel_id == null) {
            if (previous != null && state.session_id == previous.session_id) stopLocal()
            return
        }
        if (previous != null && previous.session_id != state.session_id) {
            stopLocal(); error = "The call moved to another device."
            return
        }
        if (state.channel_id != desired.id) {
            if (previous == null) return

            val moved = guildStore.allGuildChannels.value[state.channel_id] ?: Channel(state.channel_id, 2, state.guild_id)
            activeChannel = moved
            stopStreamLocal()
            connectedServer = null
            voiceGatewayManager.disconnect()
            awaitConnection(moved)
        }
        currentVoiceState = state
        selfMuted = state.self_mute
        selfDeafened = state.self_deaf
        voiceGatewayManager.setMuted(selfMuted || state.mute || state.suppress, selfDeafened || state.deaf)
        connectGateway()
        if (outgoing && ringJob == null) {
            val channelId = state.channel_id
            ringJob = scope.launch {
                if (!channelApi.ringCall(channelId)) {
                    error = "Connected, but could not ring the recipients. They can still join the call."
                }
                delay(30_000)
                if (activeChannel?.id == channelId) channelApi.stopRinging(channelId, all = true)
            }
        }
    }

    private fun connectGateway() {
        val channel = activeChannel ?: return
        val state = currentVoiceState ?: return
        val update = server ?: return
        val endpoint = update.endpoint ?: return
        if (state.channel_id != channel.id || state.session_id.isEmpty() || update == connectedServer) return
        connectedServer = update
        voiceGatewayManager.connect(endpoint, channel.guild_id ?: channel.id, channel.id, state.user_id, state.session_id, update.token)
    }

    fun clear() {
        stopLocal()
        voiceStates.clear(); calls.clear(); incomingChannelId = null; error = null
        selfMuted = false; selfDeafened = false; speakerEnabled = false
    }
}
