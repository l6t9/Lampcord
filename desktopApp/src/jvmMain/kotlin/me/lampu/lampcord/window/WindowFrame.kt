package me.lampu.lampcord.window

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.FrameWindowScope
import androidx.compose.ui.window.WindowPlacement
import androidx.compose.ui.window.WindowState
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.window.jna.structure.isWindows10OrLater

@Composable
fun FrameWindowScope.WindowFrame(
    onCloseRequest: () -> Unit,
    darkTheme: Boolean,
    state: WindowState,
    captionBarHeight: Dp = 32.dp,
    lastNormalPlacement: WindowPlacement,
    content: @Composable (
        windowInset: WindowInsets,
        captionBarInset: WindowInsets,
        onMaximized: () -> Unit,
        toggleFullscreen: () -> Unit,
    ) -> Unit,
) {
    //TODO: We should have an UiState
    val isFullscreen = false
    if (getPlatformName() != "windows" || !isWindows10OrLater()) {
        LaunchedEffect(isFullscreen, lastNormalPlacement) {
            if (isFullscreen) {
                state.placement = WindowPlacement.Fullscreen
            } else if (state.placement == WindowPlacement.Fullscreen) {
                state.placement = lastNormalPlacement
            }
        }
    }

    fun toggleFullscreen() {
        if (isFullscreen) {
            state.placement = WindowPlacement.Fullscreen
        } else {
            if (state.placement == WindowPlacement.Fullscreen) {
                state.placement = lastNormalPlacement
            }
        }
    }

    when {
        getPlatformName() == "windows" && isWindows10OrLater() -> {
            WindowsWindowFrame(
                onCloseRequest = onCloseRequest,
                content = content,
                state = state,
                captionBarHeight = captionBarHeight,
                darkTheme = darkTheme,
            )
        }

        getPlatformName() == "macos" -> {
            // Mac goes here.
            content(WindowInsets(0), WindowInsets(0), { }, { toggleFullscreen() })
        }

        // Linux
        else -> {
            Box {
                fun onMaximized() {
                    state.placement =
                        if (state.placement == WindowPlacement.Maximized || state.placement == WindowPlacement.Fullscreen) {
                            WindowPlacement.Floating
                        } else {
                            WindowPlacement.Maximized
                        }
                }
                content(WindowInsets(0), WindowInsets(0), { onMaximized() }, { toggleFullscreen() })
                if (state.placement != WindowPlacement.Fullscreen) {
                    Row(modifier = Modifier.fillMaxWidth().height(32.dp)) {
                        Box(Modifier.weight(1f))
                        CaptionButton(
                            onClick = {
                                state.isMinimized = true
                            },
                            icon = CaptionButtonIcon.Minimize,
                            isActive = true,
                            isCloseButton = false,
                            darkTheme = darkTheme,
                        )
                        CaptionButton(
                            onClick = { onMaximized() },
                            icon =
                                if (state.placement == WindowPlacement.Maximized || state.placement == WindowPlacement.Fullscreen) {
                                    CaptionButtonIcon.Restore
                                } else {
                                    CaptionButtonIcon.Maximize
                                },
                            isActive = true,
                            isCloseButton = false,
                            darkTheme = darkTheme,
                        )
                        CaptionButton(
                            icon = CaptionButtonIcon.Close,
                            onClick = onCloseRequest,
                            isActive = true,
                            isCloseButton = true,
                            darkTheme = darkTheme,
                        )
                    }
                }
            }
        }
    }
}
