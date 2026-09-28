package me.lampu.lampcord.shared.voice

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.VoiceStore

@Composable
expect fun rememberVoiceJoin(voiceStore: VoiceStore): (Channel, Boolean) -> Unit
expect fun startVoiceSession()
expect fun stopVoiceSession()
expect fun notifyIncomingCall(channelId: String?)
