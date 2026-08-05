package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
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
        "dark" -> true
        "light" -> false
        "amoled" -> true
        else -> isSystemInDarkTheme
    }

    val pureBlack = chatState.settingsStore.themeMode == "amoled" || 
            (chatState.settingsStore.themeMode == "auto" && chatState.settingsStore.pureBlack && isSystemInDarkTheme)

    LampcordTheme(
        useDarkTheme = useDarkTheme,
        pureBlack = pureBlack
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
