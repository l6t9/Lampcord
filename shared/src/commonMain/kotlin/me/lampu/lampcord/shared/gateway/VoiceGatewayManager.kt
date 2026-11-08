package me.lampu.lampcord.shared.gateway

import androidx.compose.ui.graphics.ImageBitmap
import io.ktor.client.HttpClient
import kotlinx.coroutines.flow.StateFlow
import kotlinx.serialization.json.Json

enum class VoicePhase { DISCONNECTED, CONNECTING, CONNECTED, SECURE, FAILED }

data class VoiceConnectionStatus(
    val channelId: String? = null,
    val phase: VoicePhase = VoicePhase.DISCONNECTED,
    val error: String? = null,
    val verificationCode: String = "",
    val speaking: Set<String> = emptySet()
)

expect class VoiceGatewayManager(client: HttpClient, json: Json) {
    val status: StateFlow<VoiceConnectionStatus>
    /** Latest decoded camera frame of each call participant who is sending video. */
    val videoFrames: StateFlow<Map<String, ImageBitmap>>
    /** Connection to the Go Live stream being watched; its channel ID is the stream key. */
    val streamStatus: StateFlow<VoiceConnectionStatus>
    val streamFrame: StateFlow<ImageBitmap?>
    fun connect(endpoint: String, serverId: String, channelId: String, userId: String, sessionId: String, token: String)
    fun watchStream(endpoint: String, rtcServerId: String, streamKey: String, userId: String, sessionId: String, token: String)
    fun stopStream()
    fun setMuted(muted: Boolean, deafened: Boolean)
    fun setSpeaker(enabled: Boolean)
    fun disconnect()
    fun close()
}
