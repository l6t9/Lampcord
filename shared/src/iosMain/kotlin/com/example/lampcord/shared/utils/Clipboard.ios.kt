package com.example.lampcord.shared.utils

import platform.UIKit.UIPasteboard

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> = emptyList()

actual fun setClipboardText(text: String) {
    UIPasteboard.generalPasteboard().string = text
}
