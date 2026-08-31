package me.lampu.lampcord.shared.ui.components

import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.annotation.OptIn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.ui.Alignment
import androidx.compose.ui.unit.dp
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(UnstableApi::class)
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
    onFullscreenClick: (() -> Unit)?
) {
    val context = LocalContext.current
    val exoPlayer = remember(url, loop, autoPlay) {
        ExoPlayer.Builder(context).build().apply {
            setMediaItem(MediaItem.fromUri(url))
            repeatMode = if (loop) ExoPlayer.REPEAT_MODE_ALL else ExoPlayer.REPEAT_MODE_OFF
            playWhenReady = autoPlay
            prepare()
        }
    }

    DisposableEffect(exoPlayer) {
        onDispose {
            exoPlayer.release()
        }
    }

    Box(modifier = modifier) {
        AndroidView(
            modifier = Modifier.fillMaxSize(),
            factory = {
                PlayerView(context).apply {
                    player = exoPlayer
                    useController = showControls
                    resizeMode = androidx.media3.ui.AspectRatioFrameLayout.RESIZE_MODE_FIT
                    setShowPreviousButton(false)
                    setShowNextButton(false)
                    controllerShowTimeoutMs = 3_000
                    layoutParams = FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT
                    )
                }
            },
            update = { view ->
                view.player = exoPlayer
                view.useController = showControls
            }
        )
        if (showControls && onFullscreenClick != null) {
            Surface(
                onClick = onFullscreenClick,
                modifier = Modifier.align(Alignment.BottomEnd).padding(12.dp),
                tonalElevation = 2.dp
            ) {
                Icon(Icons.Filled.Fullscreen, "Fullscreen", modifier = Modifier.padding(8.dp))
            }
        }
    }
}
