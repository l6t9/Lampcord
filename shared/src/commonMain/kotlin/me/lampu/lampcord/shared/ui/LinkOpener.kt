package me.lampu.lampcord.shared.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalUriHandler
import me.lampu.lampcord.shared.state.DiscordLinkHandler
import org.koin.compose.koinInject

@Composable
fun rememberLinkOpener(): (String) -> Unit {
    val uriHandler = LocalUriHandler.current
    val discordLinks: DiscordLinkHandler = koinInject()
    SideEffect { discordLinks.externalLinkOpener = { runCatching { uriHandler.openUri(it) } } }
    return remember(uriHandler, discordLinks) {
        { url ->
            if (!discordLinks.open(url)) {
                runCatching { uriHandler.openUri(url) }
            }
        }
    }
}