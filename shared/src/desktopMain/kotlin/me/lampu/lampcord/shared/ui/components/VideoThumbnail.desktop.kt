package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
actual fun VideoThumbnail(
    uri: String,
    contentDescription: String?,
    modifier: Modifier
) {
    ShimmerBox(modifier = modifier)
}
