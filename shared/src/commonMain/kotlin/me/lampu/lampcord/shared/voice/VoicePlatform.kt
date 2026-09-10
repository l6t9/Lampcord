package me.lampu.lampcord.shared.voice

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.VoiceStore

/** The returned action requests permission before joining. Ring only when starting a DM call. */
@Composable
expect fun rememberVoiceJoin(voiceStore: VoiceStore): (Channel, Boolean) -> Unit
expect fun startVoiceSession()
expect fun stopVoiceSession()
expect fun notifyIncomingCall(channelId: String?)
