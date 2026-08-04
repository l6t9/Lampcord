package me.lampu.lampcord.shared.utils

expect fun getClipboardFiles(): List<Pair<String, ByteArray>>

expect fun setClipboardText(text: String)
