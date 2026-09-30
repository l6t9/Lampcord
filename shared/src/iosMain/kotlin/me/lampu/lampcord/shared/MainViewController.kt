package me.lampu.lampcord.shared

import androidx.compose.ui.window.ComposeUIViewController
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlin.experimental.ExperimentalObjCName
import kotlin.experimental.ExperimentalObjCRefinement
import kotlin.native.ObjCName
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalUriHandler
import me.lampu.lampcord.shared.state.DiscordLinkHandler
import me.lampu.lampcord.shared.ui.App
import org.koin.compose.koinInject
import platform.UIKit.UIViewController

@OptIn(ExperimentalObjCRefinement::class, ExperimentalObjCName::class)
@ObjCName("MainViewController")
fun MainViewController(): UIViewController {
    LampcordPlatform.initialize()
    return ComposeUIViewController {
        App()
        InboundLinks()
    }
}

@OptIn(ExperimentalObjCRefinement::class, ExperimentalObjCName::class)
@ObjCName("handleOpenUrl")
fun handleOpenUrl(url: String) {
    pendingLinks.trySend(url)
}@Composable
private fun InboundLinks() {
    val discordLinks: DiscordLinkHandler = koinInject()
    val uriHandler = LocalUriHandler.current
    LaunchedEffect(discordLinks, uriHandler) {
        openUrlFlow().collect { url ->
            if (url.isEmpty() || discordLinks.open(url)) return@collect
            runCatching { uriHandler.openUri(url) }
        }
    }
}

private val pendingLinks = Channel<String>(Channel.BUFFERED)

fun openUrlFlow(): Flow<String> = pendingLinks.receiveAsFlow()
