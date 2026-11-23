package me.lampu.lampcord.shared.ui.components

import androidx.compose.ui.Modifier
import me.lampu.lampcord.shared.model.PendingFile

// i assume you can do something with UIPasteboard over here
actual fun Modifier.keyboardImageReceiver(onMediaReceived: (PendingFile) -> Unit) = this