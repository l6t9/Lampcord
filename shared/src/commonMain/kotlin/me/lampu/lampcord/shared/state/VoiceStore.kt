package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.SnapshotStateMap
import kotlinx.coroutines.*
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.GatewayPayload
import me.lampu.lampcord.shared.model.VoiceServerUpdate
import me.lampu.lampcord.shared.model.VoiceState

class VoiceStore(
    private val gatewayManager: GatewayManager,
    private val voiceGatewayManager: VoiceGatewayManager,
    private val json: Json,
    private val scope: CoroutineScope
) {
    var currentVoiceState by mutableStateOf<VoiceState?>(null)
    var isVoiceConnected by mutableStateOf(false)
    var voiceConnectionDuration by mutableStateOf(0L)
    private var voiceTimerJob: Job? = null
    val voiceStates = mutableStateMapOf<String, SnapshotStateMap<String, VoiceState>>() // guildId -> userId -> VoiceState

    var isVoiceChatTextVisible by mutableStateOf(false)

    fun connectToVoice(channel: Channel) {
        gatewayManager.sendVoiceStateUpdate(
            guildId = channel.guild_id,
            channelId = channel.id,
            selfMute = currentVoiceState?.self_mute ?: false,
            selfDeaf = currentVoiceState?.self_deaf ?: false,
            selfVideo = false
        )
    }

    fun disconnectFromVoice(fallbackGuildId: String? = null) {
        val guildId = currentVoiceState?.guild_id ?: fallbackGuildId
        println("Disconnecting from voice: Guild=$guildId")
        gatewayManager.sendVoiceStateUpdate(
            guildId = guildId,
            channelId = null,
            selfMute = true,
            selfDeaf = true
        )
        voiceGatewayManager.disconnect()
    }

    fun toggleVoiceMute() {
        val state = currentVoiceState ?: return
        gatewayManager.sendVoiceStateUpdate(
            guildId = state.guild_id,
            channelId = state.channel_id,
            selfMute = !state.self_mute,
            selfDeaf = state.self_deaf,
            selfVideo = state.self_video
        )
    }

    fun toggleVoiceDeaf() {
        val state = currentVoiceState ?: return
        gatewayManager.sendVoiceStateUpdate(
            guildId = state.guild_id,
            channelId = state.channel_id,
            selfMute = state.self_mute,
            selfDeaf = !state.self_deaf,
            selfVideo = state.self_video
        )
    }

    fun toggleVoiceVideo() {
        val state = currentVoiceState ?: return
        gatewayManager.sendVoiceStateUpdate(
            guildId = state.guild_id,
            channelId = state.channel_id,
            selfMute = state.self_mute,
            selfDeaf = state.self_deaf,
            selfVideo = !state.self_video
        )
    }

    fun toggleVoiceStream() {
        // selfStream is not in the VoiceStateUpdate payload
    }

    fun handleVoiceStateUpdate(payload: GatewayPayload, currentUserId: String?) {
        payload.d?.let { data ->
            try {
                val state = json.decodeFromJsonElement<VoiceState>(data)
                println("Voice State Update: User=${state.user_id}, Channel=${state.channel_id}, Guild=${state.guild_id}")
                val guildId = state.guild_id ?: "@me"
                val guildMap = voiceStates.getOrPut(guildId) { mutableStateMapOf() }
                
                if (state.channel_id == null) {
                    guildMap.remove(state.user_id)
                } else {
                    guildMap[state.user_id] = state
                }

                if (state.user_id == currentUserId) {
                    if (state.channel_id == null) {
                        currentVoiceState = null
                        isVoiceConnected = false
                        stopVoiceTimer()
                    } else {
                        val isNewConnection = currentVoiceState == null
                        currentVoiceState = state
                        isVoiceConnected = true
                        if (isNewConnection) startVoiceTimer()
                    }
                }
            } catch (e: Exception) { }
        }
    }

    fun handleVoiceServerUpdate(payload: GatewayPayload, currentUserId: String?) {
        payload.d?.let { data ->
            try {
                val update = json.decodeFromJsonElement<VoiceServerUpdate>(data)
                val userId = currentUserId ?: return
                val sessionId = currentVoiceState?.session_id ?: return
                
                if (update.endpoint != null) {
                    // voiceGatewayManager.connect(
                    //     endpoint = update.endpoint,
                    //     guildId = update.guild_id,
                    //     userId = userId,
                    //     sessionId = sessionId,
                    //     token = update.token
                    // )
                }
            } catch (e: Exception) { }
        }
    }

    private fun startVoiceTimer() {
        voiceTimerJob?.cancel()
        voiceConnectionDuration = 0L
        voiceTimerJob = scope.launch {
            while (isActive) {
                delay(1000)
                voiceConnectionDuration++
            }
        }
    }

    private fun stopVoiceTimer() {
        voiceTimerJob?.cancel()
        voiceTimerJob = null
        voiceConnectionDuration = 0L
    }

    fun clear() {
        currentVoiceState = null
        isVoiceConnected = false
        stopVoiceTimer()
        voiceStates.clear()
    }
}
