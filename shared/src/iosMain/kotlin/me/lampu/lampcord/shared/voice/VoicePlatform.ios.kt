package me.lampu.lampcord.shared.voice

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.VoiceStore

@Composable
actual fun rememberVoiceJoin(voiceStore: VoiceStore): (Channel, Boolean) -> Unit = { _, _ ->
    voiceStore.error = "Voice calls are currently available on Android and desktop only."
}
actual fun startVoiceSession(): Unit = error("Voice is unavailable on iOS")
actual fun stopVoiceSession() = Unit
actual fun notifyIncomingCall(channelId: String?) = Unit
