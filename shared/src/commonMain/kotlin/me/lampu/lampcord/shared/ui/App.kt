package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import coil3.ImageLoader
import coil3.PlatformContext as CoilPlatformContext
import coil3.compose.setSingletonImageLoaderFactory
import coil3.memory.MemoryCache as CoilMemoryCache
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.request.crossfade as coilCrossfade
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

    setSingletonImageLoaderFactory { context ->
        newCoilImageLoader(context)
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
        .memoryCache { MemoryCache.Builder(context).maxSizePercent(0.15).build() }
        .components {
            add(apngDecoderFactory())
        }
        .build()
}

/** Coil handles still images; Sketch is kept only for animated decorations, since Coil 3 animates on Android only. */
fun newCoilImageLoader(context: CoilPlatformContext): ImageLoader =
    ImageLoader.Builder(context)
        .components {
            add(KtorNetworkFetcherFactory())
        }
        .memoryCache {
            CoilMemoryCache.Builder()
                .maxSizePercent(context, 0.20)
                .build()
        }
        .coilCrossfade(true)
        .build()
