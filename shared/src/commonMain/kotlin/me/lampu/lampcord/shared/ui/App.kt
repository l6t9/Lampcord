package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.compose.setSingletonImageLoaderFactory
import coil3.request.crossfade
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.theme.LampcordTheme
import org.koin.compose.koinInject

@Composable
fun App() {
    setSingletonImageLoaderFactory { context ->
        newImageLoader(context)
    }
    
    val chatState: ChatState = koinInject()
    val isSystemInDarkTheme = androidx.compose.foundation.isSystemInDarkTheme()
    
    val useDarkTheme = when (chatState.settingsStore.themeMode) {
        me.lampu.lampcord.shared.settings.ThemeMode.DARK -> true
        me.lampu.lampcord.shared.settings.ThemeMode.LIGHT -> false
        else -> isSystemInDarkTheme
    }
    
    val pureBlack = chatState.settingsStore.pureBlack && useDarkTheme

    val seedColor = remember(chatState.settingsStore.accentColor) {
        try {
            Color(chatState.settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
        } catch (e: Exception) {
            Color(0xFF6750A4)
        }
    }

    LampcordTheme(
        useDarkTheme = useDarkTheme,
        pureBlack = pureBlack,
        paletteStyle = chatState.settingsStore.themePaletteStyle,
        useMaterialYou = chatState.settingsStore.materialYou,
        appFont = chatState.settingsStore.appFont,
        fontScale = chatState.settingsStore.fontScale,
        customFontPath = chatState.settingsStore.customFontPath,
        seedColor = seedColor
    ) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 0.dp
        ) {
            MainScreen(chatState)
        }
    }
}

fun newImageLoader(context: PlatformContext): ImageLoader {
    return ImageLoader.Builder(context)
        .crossfade(true)
        .build()
}
