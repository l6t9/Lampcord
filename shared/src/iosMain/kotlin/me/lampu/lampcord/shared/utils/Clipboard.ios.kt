package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import platform.UIKit.UIPasteboard

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> = emptyList()

actual fun setClipboardText(text: String) {
    UIPasteboard.generalPasteboard().string = text
}

@Composable
actual fun ProvideClipboard() = Unit
