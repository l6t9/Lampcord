package me.lampu.lampcord.shared.gateway

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
    fun connect(endpoint: String, serverId: String, channelId: String, userId: String, sessionId: String, token: String)
    fun setMuted(muted: Boolean, deafened: Boolean)
    fun setSpeaker(enabled: Boolean)
    fun disconnect()
    fun close()
}
