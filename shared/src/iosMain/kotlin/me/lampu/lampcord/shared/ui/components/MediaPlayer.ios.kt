package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.interop.UIKitView
import kotlinx.cinterop.ExperimentalForeignApi
import me.lampu.lampcord.shared.utils.Logging
import platform.AVFAudio.AVAudioSession
import platform.AVFAudio.AVAudioSessionCategoryPlayback
import platform.AVFAudio.setActive
import platform.AVFoundation.AVPlayer
import platform.AVFoundation.AVPlayerItem
import platform.AVFoundation.AVPlayerItemDidPlayToEndTimeNotification
import platform.AVFoundation.AVPlayerItemFailedToPlayToEndTimeNotification
import platform.AVFoundation.AVURLAsset
import platform.AVFoundation.currentItem
import platform.AVFoundation.pause
import platform.AVFoundation.play
import platform.AVFoundation.replaceCurrentItemWithPlayerItem
import platform.AVFoundation.seekToTime
import platform.AVFoundation.volume
import platform.AVKit.AVPlayerViewController
import platform.CoreMedia.CMTimeMakeWithSeconds
import platform.Foundation.NSNotificationCenter
import platform.Foundation.NSURL

@OptIn(ExperimentalForeignApi::class)
private fun newPlayer(): AVPlayer {
    // Playback has to keep running with the ringer switch silenced. An interruption resets the
    // session, so this is re-asserted for every player rather than once per process.
    runCatching {
        AVAudioSession.sharedInstance().setCategory(AVAudioSessionCategoryPlayback, null)
        AVAudioSession.sharedInstance().setActive(true, null)
    }.onFailure { Logging.w(TAG, "Unable to configure the audio session", it) }
    return AVPlayer()
}

@OptIn(ExperimentalForeignApi::class)
private fun seekToStart(player: AVPlayer) {
    val zero = CMTimeMakeWithSeconds(0.0, TIMELINE_PRECISION)
    player.seekToTime(zero, toleranceBefore = zero, toleranceAfter = zero)
}

@OptIn(ExperimentalForeignApi::class)
@Composable
private fun rememberPlayer(url: String, loop: Boolean, muted: Boolean): AVPlayer? {
    val nsUrl = remember(url) { NSURL.URLWithString(url) }
    val player = remember(nsUrl) { nsUrl?.let { newPlayer() } }

    LaunchedEffect(player, nsUrl) {
        if (player == null) return@LaunchedEffect
        player.replaceCurrentItemWithPlayerItem(AVPlayerItem(asset = AVURLAsset(uRL = nsUrl!!, options = null)))
    }

    LaunchedEffect(player, muted) {
        if (player == null) return@LaunchedEffect
        player.volume = if (muted) 0f else 1f
    }

    DisposableEffect(player, loop) {
        if (player == null) {
            onDispose { }
        } else {
            val center = NSNotificationCenter.defaultCenter
            val endObserver = if (loop) {
                center.addObserverForName(AVPlayerItemDidPlayToEndTimeNotification, null, null) {
                    seekToStart(player)
                    player.play()
                }
            } else {
                null
            }
            val failureObserver = center.addObserverForName(
                AVPlayerItemFailedToPlayToEndTimeNotification, null, null
            ) { notification ->
                val reason = notification?.userInfo?.get("AVPlayerItemFailedToPlayToEndTimeErrorKey")
                Logging.w(TAG, "Playback failed: $reason")
            }
            onDispose {
                endObserver?.let { center.removeObserver(it) }
                center.removeObserver(failureObserver)
                player.pause()
                player.replaceCurrentItemWithPlayerItem(null)
            }
        }
    }

    return player
}

@OptIn(ExperimentalForeignApi::class)
@Composable
actual fun AudioPlayer(
    url: String,
    modifier: Modifier,
    autoPlay: Boolean,
) {
    val player = rememberPlayer(url, loop = false, muted = false)

    LaunchedEffect(player, autoPlay) {
        if (player == null) return@LaunchedEffect
        if (autoPlay) player.play() else player.pause()
    }
}

@OptIn(ExperimentalForeignApi::class)
@Composable
private fun VideoSurface(player: AVPlayer?, showControls: Boolean, modifier: Modifier) {
    UIKitView(
        factory = {
            AVPlayerViewController().apply {
                this.player = player
                showsPlaybackControls = showControls
                allowsPictureInPicturePlayback = !showControls
            }.view
        },
        modifier = modifier
    )
}

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
    onFullscreenClick: (() -> Unit)?,
) {
    val player = rememberPlayer(url, loop, muted = compact)

    LaunchedEffect(player, autoPlay) {
        if (player == null) return@LaunchedEffect
        if (autoPlay) player.play() else player.pause()
    }

    VideoSurface(player, showControls, modifier)
}

private const val TAG = "media"
private const val TIMELINE_PRECISION = 600