package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import me.lampu.lampcord.shared.model.PendingFile

@Composable
expect fun VoiceMessageRecorder(
    enabled: Boolean,
    buttonSize: Dp,
    iconSize: Dp,
    modifier: Modifier = Modifier,
    onRecordingReady: (PendingFile) -> Unit,
)
