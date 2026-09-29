package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable

expect fun getClipboardFiles(): List<Pair<String, ByteArray>>

expect fun setClipboardText(text: String)

@Composable
expect fun ProvideClipboard()
