package me.lampu.lampcord.shared.voice

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.VoiceStore

@Composable
actual fun rememberVoiceJoin(voiceStore: VoiceStore): (Channel, Boolean) -> Unit = voiceStore::connectToVoice
actual fun startVoiceSession() = Unit
actual fun stopVoiceSession() = Unit
actual fun notifyIncomingCall(channelId: String?) {
    if (channelId != null) java.awt.Toolkit.getDefaultToolkit().beep()
}
