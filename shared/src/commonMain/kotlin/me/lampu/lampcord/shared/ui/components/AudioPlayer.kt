package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
expect fun AudioPlayer(
    url: String,
    modifier: Modifier = Modifier,
    autoPlay: Boolean = false,
)
