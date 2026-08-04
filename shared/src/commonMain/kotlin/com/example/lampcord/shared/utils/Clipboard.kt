package com.example.lampcord.shared.utils

expect fun getClipboardFiles(): List<Pair<String, ByteArray>>

expect fun setClipboardText(text: String)
