package me.lampu.lampcord.shared.utils

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> = emptyList() // Android handles this via system UI usually

actual fun setClipboardText(text: String) {
    // TODO: Implementation with context
}
