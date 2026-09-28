package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

enum class WindowWidthBreakpoint {
    COMPACT, MEDIUM, EXPANDED, LARGE, EXTRA_LARGE
}

@Composable
fun calculateWidthBreakpoint(width: androidx.compose.ui.unit.Dp): WindowWidthBreakpoint {
    return when {
        width < 600.dp -> WindowWidthBreakpoint.COMPACT
        width < 840.dp -> WindowWidthBreakpoint.MEDIUM
        width < 1200.dp -> WindowWidthBreakpoint.EXPANDED
        width < 1600.dp -> WindowWidthBreakpoint.LARGE
        else -> WindowWidthBreakpoint.EXTRA_LARGE
    }
}

@Composable
fun MainAdaptiveScaffold(
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    userStore: UserStore = koinInject(),
    voiceStore: VoiceStore = koinInject()
) {
    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val widthBreakpoint = calculateWidthBreakpoint(maxWidth)
        val platform = getPlatformName()
        val isDesktop = platform == "desktop" || platform == "macos" || platform == "windows" || platform == "linux"

        when (widthBreakpoint) {
            WindowWidthBreakpoint.COMPACT -> {
                MobileBaseplate(navigationStore, profileStore, userStore, voiceStore)
            }
            WindowWidthBreakpoint.MEDIUM -> {
                if (isDesktop) {
                    DesktopBaseplate(navigationStore, profileStore, voiceStore, widthBreakpoint)
                } else {
                    MobileBaseplate(navigationStore, profileStore, userStore, voiceStore)
                }
            }
            WindowWidthBreakpoint.EXPANDED, WindowWidthBreakpoint.LARGE, WindowWidthBreakpoint.EXTRA_LARGE -> {
                DesktopBaseplate(navigationStore, profileStore, voiceStore, widthBreakpoint)
            }
        }
    }
}
