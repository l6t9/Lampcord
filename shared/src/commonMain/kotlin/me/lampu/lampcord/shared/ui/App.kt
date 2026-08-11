package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.compose.setSingletonImageLoaderFactory
import coil3.request.crossfade
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.theme.LampcordTheme
import org.jetbrains.compose.resources.ExperimentalResourceApi
import org.koin.compose.koinInject

@OptIn(ExperimentalResourceApi::class)
@Composable
fun App() {
    val settingsStore: SettingsStore = koinInject()
    
    val useDarkTheme = when (settingsStore.themeMode) {
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
        ThemeMode.AUTO -> isSystemInDarkTheme()
    }

    val pureBlack = settingsStore.pureBlack && useDarkTheme

    val seedColor = remember(settingsStore.accentColor) {
        try {
            Color(settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
        } catch (e: Exception) {
            Color(0xFF6750A4)
        }
    }

    setSingletonImageLoaderFactory { context ->
        newImageLoader(context)
    }

    LampcordTheme(
        useDarkTheme = useDarkTheme,
        pureBlack = pureBlack,
        seedColor = seedColor,
        paletteStyle = settingsStore.themePaletteStyle,
        useMaterialYou = settingsStore.materialYou,
        appFont = settingsStore.appFont,
        fontScale = settingsStore.fontScale,
        customFontPath = settingsStore.customFontPath,
    ) {
        MainScreen()
    }
}

fun newImageLoader(context: PlatformContext): ImageLoader {
    return ImageLoader.Builder(context)
        .crossfade(true)
        .build()
}
