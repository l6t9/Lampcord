package me.lampu.lampcord.shared.ui

import me.lampu.lampcord.shared.ui.components.captcha.CaptchaHost
import me.lampu.lampcord.shared.ui.components.guilds.PendingInviteHost
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import coil3.ImageLoader
import coil3.PlatformContext as CoilPlatformContext
import coil3.annotation.ExperimentalCoilApi
import coil3.compose.setSingletonImageLoaderFactory
import coil3.disk.DiskCache
import coil3.memory.MemoryCache as CoilMemoryCache
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.request.crossfade as coilCrossfade
import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.header
import io.ktor.http.HttpHeaders
import me.lampu.lampcord.shared.api.httpClientEngine
import me.lampu.lampcord.shared.imaging.platformImageDecoders
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.theme.LampcordTheme
import me.lampu.lampcord.shared.utils.ProvideClipboard
import me.lampu.lampcord.shared.utils.getAppStoragePath
import me.lampu.lampcord.shared.utils.getPlatformName
import okio.Path.Companion.toPath
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

    val languageTag = settingsStore.language
        .takeUnless { it == me.lampu.lampcord.shared.i18n.LANGUAGE_SYSTEM }

    val seedColor = remember(settingsStore.accentColor) {
        try {
            Color(settingsStore.accentColor.removePrefix("#").toLong(16) or 0xFF000000)
        } catch (_: Exception) {
            Color(0xFF6750A4)
        }
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
        me.lampu.lampcord.shared.i18n.ProvideAppLocale(languageTag) {
            ProvideClipboard()
            MainScreen()
            UpdatePrompt()
            PendingInviteHost()
            CaptchaHost()
        }
    }
}

@OptIn(ExperimentalCoilApi::class)
fun newCoilImageLoader(context: CoilPlatformContext): ImageLoader {
    val platform = getPlatformName()
    val isMobile = platform == "android" || platform == "ios"
    val userAgent = if (isMobile) {
        if (platform == "android") "Discord-Android/341200;RNA" else "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
    } else {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
    }

    val imageHttpClient = HttpClient(httpClientEngine()) {
        install(HttpTimeout) {
            requestTimeoutMillis = 15000
            connectTimeoutMillis = 10000
            socketTimeoutMillis = 15000
        }
        defaultRequest {
            header(HttpHeaders.UserAgent, userAgent)
            header(HttpHeaders.Accept, "image/webp,image/apng,image/png,image/jpeg,image/svg+xml,image/*,*/*;q=0.8")
            header("Origin", "https://discord.com")
            header("Referer", "https://discord.com/")
        }
    }

    return ImageLoader.Builder(context)
        .components {
            add(KtorNetworkFetcherFactory(imageHttpClient))

            platformImageDecoders().forEach { add(it) }
        }
        .diskCache {
            DiskCache.Builder()
                .directory("${getAppStoragePath()}/image_cache".toPath())
                .maxSizeBytes(256L * 1024 * 1024)
                .build()
        }
        .memoryCache {
            CoilMemoryCache.Builder()
                .maxSizeBytes(32 * 1024 * 1024)
                .build()
        }
        .coilCrossfade(true)
        .build()
}
