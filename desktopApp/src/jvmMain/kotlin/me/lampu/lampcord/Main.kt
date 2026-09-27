package me.lampu.lampcord

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.snap
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.*
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.WindowPlacement
import androidx.compose.ui.window.WindowPosition
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.rpc.DesktopRPCServer
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.ui.App
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.reloadTrigger
import me.lampu.lampcord.ui.WaylandDensityProvider
import me.lampu.lampcord.utils.WaylandScale
import me.lampu.lampcord.window.WindowFrame
import org.koin.compose.koinInject
import org.koin.core.context.GlobalContext.get
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin

fun main() {
    // Read before building the HTTP client.
    Logging.debugEnabled = Settings.shared.verboseLogging

    WaylandScale.detectAndApply()

    fun initApp() {
        stopKoin()
        startKoin {
            modules(appModule, me.lampu.lampcord.shared.di.desktopNotificationModule)
        }
    }

    initApp()

    val rpcServer = DesktopRPCServer(get().get())
    rpcServer.start()

    val isLinux = getPlatformName() == "linux"
    val isMac = getPlatformName() == "macos"

    if (WaylandScale.isWayland() && !Settings.shared.waylandDefaultFrameApplied) {
        Settings.shared.enableSystemWindowFrame = true
        Settings.shared.waylandDefaultFrameApplied = true
    }
    if (isMac && !Settings.shared.macDefaultFrameApplied) {
        Settings.shared.enableSystemWindowFrame = true
        Settings.shared.macDefaultFrameApplied = true
    }
    val useSystemWindowFrame = Settings.shared.enableSystemWindowFrame

    application {
        val reloadKey by reloadTrigger.collectAsState()
        
        LaunchedEffect(reloadKey) {
            if (reloadKey > 0) {
                initApp()
            }
        }

        val settingsStore: SettingsStore = koinInject()
        
        val seedColorString = settingsStore.accentColor
        val targetSeedColor = remember(seedColorString) {
            try {
                Color(seedColorString.removePrefix("#").toLong(16) or 0xFF000000)
            } catch (_: Exception) {
                Color(0xFF6750A4)
            }
        }
        
        val seedColor by animateColorAsState(
            targetValue = targetSeedColor,
            animationSpec = if (Settings.shared.reduceMotion) snap() else androidx.compose.animation.core.spring()
        )
        
        val discordPainter = rememberVectorPainter(Icons.Brand.Discord)
        val dynamicIcon = remember(seedColor) {
            object : Painter() {
                override val intrinsicSize: Size = Size(256f, 256f)
                override fun DrawScope.onDraw() {
                    val cornerRadius = size.width * 0.25f
                    drawRoundRect(
                        color = seedColor,
                        size = size,
                        cornerRadius = CornerRadius(cornerRadius, cornerRadius)
                    )
                    with(discordPainter) {
                        val logoSize = size * 0.62f
                        val offset = Offset((size.width - logoSize.width) / 2f, (size.height - logoSize.height) / 2f)
                        translate(offset.x, offset.y) {
                            draw(logoSize, colorFilter = ColorFilter.tint(Color.White))
                        }
                    }
                }
            }
        }

        val savedWidth = 1200
        val savedHeight = 800

        val windowPlacement = WindowPlacement.Floating
        val windowState =
            rememberWindowState(
                placement = windowPlacement,
                position = WindowPosition.PlatformDefault,
                size = DpSize(savedWidth.dp, savedHeight.dp),
            )
        val lastNormalPlacement = remember { mutableStateOf(windowPlacement) }

        fun onClose() {
            exitApplication()
        }

        Window(
            onCloseRequest = { onClose() },
            title = "Lampcord",
            icon = dynamicIcon,
            undecorated = isLinux && !useSystemWindowFrame,
            transparent = false,
        ) {
            DisposableEffect(useSystemWindowFrame) {
                if (isMac && useSystemWindowFrame) {
                    try {
                        val rootPane = window.rootPane
                        rootPane.putClientProperty("apple.awt.fullWindowContent", true)
                        rootPane.putClientProperty("apple.awt.transparentTitleBar", true)
                        rootPane.putClientProperty("apple.awt.windowTitleVisible", false)
                        rootPane.putClientProperty("apple.awt.draggableWindowBackground", false)
                    } catch (_: Exception) {
                    }
                }
                onDispose { }
            }

            WindowFrame(
                onCloseRequest = { onClose() },
                lastNormalPlacement = lastNormalPlacement.value,
                darkTheme = when (settingsStore.themeMode) {
                    ThemeMode.LIGHT -> false
                    ThemeMode.DARK -> true
                    ThemeMode.AUTO -> isSystemInDarkTheme()
                },
                state = windowState,
            ) { _, contentInset, onMaximized, toggleFullscreen ->
                key(reloadKey) {
                    WaylandDensityProvider {
                        App()
                    }
                }
            }
        }
    }
}
