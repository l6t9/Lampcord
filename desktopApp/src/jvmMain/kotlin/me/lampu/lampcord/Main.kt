package me.lampu.lampcord

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.snap
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.runtime.rememberCoroutineScope
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
import androidx.compose.ui.window.WindowPlacement
import androidx.compose.ui.window.WindowPosition
import androidx.compose.ui.window.rememberWindowState
import dev.nucleusframework.application.LocalNucleusWindow
import dev.nucleusframework.application.Window
import dev.nucleusframework.application.nucleusApplication
import dev.nucleusframework.composenativetray.tray.api.Tray
import dev.nucleusframework.energymanager.EnergyManager
import dev.nucleusframework.notification.common.NotificationManager
import dev.nucleusframework.systemcolor.systemAccentColor
import dev.nucleusframework.updater.NucleusUpdater
import dev.nucleusframework.updater.UpdateResult
import dev.nucleusframework.updater.provider.GitHubProvider
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.rpc.DesktopRPCServer
import me.lampu.lampcord.shared.state.AppVisibilityStore
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.ui.App
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Logging
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.reloadTrigger
import org.koin.compose.koinInject
import org.koin.core.context.GlobalContext.get
import org.koin.core.context.GlobalContext.getOrNull
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin

@OptIn(FlowPreview::class)
fun main() {
    Logging.debugEnabled = Settings.shared.verboseLogging

    fun initApp() {
        runCatching { getOrNull()?.getOrNull<SessionManager>()?.shutdown() }
        stopKoin()
        startKoin {
            modules(appModule, me.lampu.lampcord.shared.di.desktopNotificationModule)
        }
    }

    initApp()

    NotificationManager.initialize()

    val rpcServer = DesktopRPCServer(get().get())
    rpcServer.start()

    val isLinux = getPlatformName() == "linux"
    val isHyprland = isLinux && (
        System.getenv("HYPRLAND_INSTANCE_SIGNATURE") != null ||
            System.getenv("XDG_CURRENT_DESKTOP")?.contains("hyprland", ignoreCase = true) == true
        )

    nucleusApplication {
        val reloadKey by reloadTrigger.collectAsState()

        LaunchedEffect(reloadKey) {
            if (reloadKey > 0) {
                initApp()
            }
        }

        val settingsStore: SettingsStore = koinInject()

        val seedColorString = settingsStore.accentColor
        val systemAccent = systemAccentColor()
        val targetSeedColor = remember(seedColorString, systemAccent) {
            try {
                if (seedColorString.isNotBlank() && seedColorString != "#6750A4") {
                    Color(seedColorString.removePrefix("#").toLong(16) or 0xFF000000)
                } else {
                    systemAccent ?: Color(0xFF6750A4)
                }
            } catch (_: Exception) {
                systemAccent ?: Color(0xFF6750A4)
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
                    with(discordPainter) {
                        draw(size, colorFilter = ColorFilter.tint(seedColor))
                    }
                }
            }
        }

        val savedWidth = Settings.shared.windowWidth
        val savedHeight = Settings.shared.windowHeight

        val windowPlacement = when (Settings.shared.windowPlacement) {
            "maximized" -> WindowPlacement.Maximized
            "fullscreen" -> WindowPlacement.Fullscreen
            else -> WindowPlacement.Floating
        }
        val windowState =
            rememberWindowState(
                placement = windowPlacement,
                position = WindowPosition.PlatformDefault,
                size = DpSize(savedWidth.dp, savedHeight.dp),
            )

        LaunchedEffect(windowState) {
            snapshotFlow {
                Triple(windowState.size.width.value, windowState.size.height.value, windowState.placement)
            }
                .debounce(500)
                .collect { (width, height, placement) ->
                    if (placement == WindowPlacement.Floating) {
                        Settings.shared.windowWidth = width.toInt().coerceAtLeast(640)
                        Settings.shared.windowHeight = height.toInt().coerceAtLeast(480)
                    }
                    Settings.shared.windowPlacement = when (placement) {
                        WindowPlacement.Maximized -> "maximized"
                        WindowPlacement.Fullscreen -> "fullscreen"
                        else -> "floating"
                    }
                }
        }

        val isDark = when (settingsStore.themeMode) {
            ThemeMode.LIGHT -> false
            ThemeMode.DARK -> true
            ThemeMode.AUTO -> isSystemInDarkTheme()
        }

        MaterialTheme(
            colorScheme = if (isDark) MaterialTheme.colorScheme.copy(
                background = Color(0xFF121212),
                surface = Color(0xFF121212)
            ) else MaterialTheme.colorScheme
        ) {
            val scope = rememberCoroutineScope()
            val windowColorScheme = MaterialTheme.colorScheme
            val windowTypography = MaterialTheme.typography
            val windowShapes = MaterialTheme.shapes

            Tray(
                icon = dynamicIcon,
                tooltip = "Lampcord",
                primaryAction = { scope.launch { windowState.isMinimized = false } }
            ) {
                Item("Open") { scope.launch { windowState.isMinimized = false } }
                Divider()
                Item("Quit") { exitApplication() }
            }

            val updater = remember {
                NucleusUpdater {
                    provider = GitHubProvider("l6t9", "Lampcord")
                    differentialDownload = true
                }
            }

            LaunchedEffect(updater) {
                when (val result = updater.checkForUpdates()) {
                    is UpdateResult.Available ->
                        Logging.i("Lampcord", "Update available: ${result.info.version}")
                    is UpdateResult.Error ->
                        Logging.i("Lampcord", "Update check failed: ${result.exception.message}")
                    is UpdateResult.NotAvailable ->
                        Logging.i("Lampcord", "Lampcord is up to date (${updater.currentVersion})")
                }
            }

            Window(
                onCloseRequest = { exitApplication() },
                title = "Lampcord",
                icon = dynamicIcon,
                state = windowState,
                undecorated = isHyprland,
            ) {
                val nucleusWindow = LocalNucleusWindow.current
                LaunchedEffect(nucleusWindow) {
                    nucleusWindow.focusFlow.collect { AppVisibilityStore.setVisible(it) }
                }
                MaterialTheme(
                    colorScheme = windowColorScheme,
                    typography = windowTypography,
                    shapes = windowShapes,
                ) {
                    DisposableEffect(Unit) {
                        EnergyManager.keepAwake()
                        onDispose { EnergyManager.releaseAwake() }
                    }
                    App()
                }
            }
        }
    }
}
