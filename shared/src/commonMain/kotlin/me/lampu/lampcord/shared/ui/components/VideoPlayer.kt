package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
expect fun VideoPlayer(
    url: String,
    modifier: Modifier = Modifier,
    loop: Boolean = false,
    showControls: Boolean = true,
    title: String? = null,
    subtitle: String? = null,
    compact: Boolean = false,
    autoPlay: Boolean = true,
    showSeekBar: Boolean = true,
    onFullscreenClick: (() -> Unit)? = null
)
