package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import com.github.panpf.sketch.PlatformContext
import com.github.panpf.sketch.SingletonSketch
import com.github.panpf.sketch.Sketch
import com.github.panpf.sketch.cache.MemoryCache
import me.lampu.lampcord.shared.image.apngDecoderFactory
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
        } catch (_: Exception) {
            Color(0xFF6750A4)
        }
    }

    SingletonSketch.setSafe { context ->
        newSketch(context)
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

fun newSketch(context: PlatformContext): Sketch {
    return Sketch.Builder(context)
        // Limit the heap cache to 15%.
        .memoryCache { MemoryCache.Builder(context).maxSizePercent(0.15).build() }
        .components {
            add(apngDecoderFactory())
        }
        .build()
}
