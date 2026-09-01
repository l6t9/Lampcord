package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.interop.UIKitView
import kotlinx.cinterop.ExperimentalForeignApi
import platform.AVFoundation.AVPlayer
import platform.AVFoundation.AVPlayerLayer
import platform.AVKit.AVPlayerViewController
import platform.Foundation.NSURL
import platform.UIKit.UIView

@OptIn(ExperimentalForeignApi::class)
@Composable
actual fun VideoPlayer(
    url: String,
    modifier: Modifier,
    loop: Boolean,
    showControls: Boolean,
    title: String?,
    subtitle: String?,
    compact: Boolean,
    autoPlay: Boolean,
    showSeekBar: Boolean,
    onFullscreenClick: (() -> Unit)?
) {
    val nsUrl = remember(url) { NSURL.URLWithString(url) }
    val player = remember(nsUrl) { nsUrl?.let { AVPlayer.playerWithURL(it) } }

    if (player != null) {
        if (!autoPlay) player.pause()
        UIKitView(
            factory = {
                val playerViewController = AVPlayerViewController()
                playerViewController.player = player
                playerViewController.view
            },
            modifier = modifier,
            update = { _ ->
                // Native controls will handle playback
            }
        )
    }
}
