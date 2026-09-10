package me.lampu.lampcord.shared.gateway

import io.ktor.client.HttpClient
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.serialization.json.Json

actual class VoiceGatewayManager actual constructor(client: HttpClient, json: Json) {
    private val state = MutableStateFlow(VoiceConnectionStatus())
    actual val status: StateFlow<VoiceConnectionStatus> = state
    actual fun connect(endpoint: String, serverId: String, channelId: String, userId: String, sessionId: String, token: String) {
        state.value = VoiceConnectionStatus(channelId, VoicePhase.FAILED, "Voice is unavailable on iOS")
    }
    actual fun setMuted(muted: Boolean, deafened: Boolean) = Unit
    actual fun setSpeaker(enabled: Boolean) = Unit
    actual fun disconnect() { state.value = VoiceConnectionStatus() }
}
