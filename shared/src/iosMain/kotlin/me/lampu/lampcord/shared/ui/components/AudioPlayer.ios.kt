package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.interop.UIKitView
import kotlinx.cinterop.ExperimentalForeignApi
import platform.AVFoundation.AVPlayer
import platform.AVKit.AVPlayerViewController
import platform.Foundation.NSURL

@OptIn(ExperimentalForeignApi::class)
@Composable
actual fun AudioPlayer(
    url: String,
    modifier: Modifier,
    autoPlay: Boolean,
) {
    val nsUrl = remember(url) { NSURL.URLWithString(url) }
    val player = remember(nsUrl) { nsUrl?.let { AVPlayer.playerWithURL(it) } }

    if (player != null) {
        LaunchedEffect(player, autoPlay) {
            if (autoPlay) player.play() else player.pause()
        }

        DisposableEffect(player) {
            onDispose { player.pause() }
        }

        UIKitView(
            factory = {
                AVPlayerViewController().apply {
                    this.player = player
                    showsPlaybackControls = true
                }.view
            },
            modifier = modifier,
        )
    }
}
