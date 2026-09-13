package me.lampu.lampcord.shared.ui

import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.baseplates.*
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun MainScreen(
    navigationStore: NavigationStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    errorStore: AppErrorStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val reduceMotion = Settings.shared.reduceMotion

        val loadingMessage = remember(navigationStore.isConnecting, messageStore.loadingMessages.size) {
            if (navigationStore.isConnecting) {
                if (messageStore.loadingMessages.isNotEmpty()) {
                    messageStore.loadingMessages.random()
                } else "Connecting to Discord..."
            } else ""
        }

        val hasVoice = voiceStore.activeChannel != null
        Column(Modifier.fillMaxSize().then(if (hasVoice) Modifier.statusBarsPadding() else Modifier)) {
            if (hasVoice) VoiceConnectionPanel(voiceStore)
            AnimatedContent(
                modifier = Modifier.weight(1f),
                targetState = navigationStore.isConnected to navigationStore.isConnecting,
                transitionSpec = {
                    if (reduceMotion) {
                        EnterTransition.None togetherWith ExitTransition.None
                    } else {
                        fadeIn(tween(300)).togetherWith(fadeOut(tween(300)))
                    }
                },
                label = "MainScreenContentTransition"
            ) { (isConnected, isConnecting) ->
                when {
                    isConnected -> {
                        MainAdaptiveScaffold(
                            navigationStore = navigationStore,
                            profileStore = profileStore,
                            userStore = userStore,
                            voiceStore = voiceStore
                        )
                    }
                    isConnecting || me.lampu.lampcord.shared.settings.Settings.shared.discordToken.isNotBlank() -> {
                        Box(
                            modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface),
                            contentAlignment = Alignment.Center
                        ) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                ContainedLoadingIndicator(modifier = Modifier.size(48.dp))
                                Spacer(Modifier.height(16.dp))
                                Text(
                                    text = loadingMessage,
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                    else -> {
                        LoginScreen(
                            onLoginSuccess = { /* No-op, navigationStore updates will trigger re-compose */ }
                        )
                    }
                }
            }
        }
        if (navigationStore.isConnected) VoiceCallDialogs()

        // Global Overlays
        if (navigationStore.isConnected) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
                InAppNotificationHost()
            }
        }

        val errors = errorStore.errors
        if (errors.isNotEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(16.dp),
                contentAlignment = Alignment.BottomCenter
            ) {
                Surface(
                    color = MaterialTheme.colorScheme.errorContainer,
                    contentColor = MaterialTheme.colorScheme.onErrorContainer,
                    shape = RoundedCornerShape(8.dp),
                    tonalElevation = 4.dp
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(errors.first().message, modifier = Modifier.weight(1f))
                        TextButton(onClick = { errorStore.clear() }) {
                            Text("Dismiss")
                        }
                    }
                }
            }
        }
        
        // Quick Switcher (Ctrl+K / Cmd+K)
        if (navigationStore.isQuickSwitcherVisible) {
            QuickSwitcher(
                onDismiss = { navigationStore.isQuickSwitcherVisible = false }
            )
        }
        
        // Forwarding Dialog
        navigationStore.forwardingMessage?.let { message ->
            ForwardDialog(
                message = message,
                onDismiss = { navigationStore.forwardingMessage = null }
            )
        }
    }
}
