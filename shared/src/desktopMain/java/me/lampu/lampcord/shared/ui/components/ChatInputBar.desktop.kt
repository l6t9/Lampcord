package me.lampu.lampcord.shared.ui.components

import androidx.compose.ui.Modifier
import me.lampu.lampcord.shared.model.PendingFile

actual fun Modifier.keyboardImageReceiver(onMediaReceived: (PendingFile) -> Unit) = this