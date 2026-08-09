package me.lampu.lampcord.shared.ui

import androidx.compose.animation.*
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.input.key.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.baseplates.DesktopBaseplate
import me.lampu.lampcord.shared.ui.baseplates.MobileBaseplate
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.ui.components.*
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MainScreen(
    chatState: ChatState
) {
    val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()
    val snackbarHostState = remember { SnackbarHostState() }

    LaunchedEffect(chatState.errorStore.errors.size) {
        val error = chatState.errorStore.errors.firstOrNull()
        if (error != null) {
            snackbarHostState.showSnackbar(
                message = error.message,
                duration = SnackbarDuration.Short
            )
            chatState.errorStore.consumeError(error)
        }
    }

    Scaffold(
        snackbarHost = { SnackbarHost(snackbarHostState) },
        modifier = Modifier.fillMaxSize()
    ) { padding ->
        AnimatedContent(
            targetState = chatState.isConnected to chatState.isConnecting,
            modifier = Modifier.fillMaxSize().padding(padding).background(MaterialTheme.colorScheme.surface),
            transitionSpec = {
                fadeIn(quickEffectsSpec) togetherWith fadeOut(quickEffectsSpec)
            },
            label = "MainScreenTransition"
        ) { (isConnected, isConnecting) ->
            if (!isConnected) {
                if (isConnecting) {
                    println(chatState.currentUser)
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Column(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            ContainedLoadingIndicator()
                            Spacer(Modifier.height(8.dp))
                            Row(
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Text(
                                    text = when(DateTimeUtils.getCurrentTime24H().split(":")[0].toInt()){
                                        0, 1, 2, 3, 4, 5 -> "Productive night I see, "
                                        6, 7, 8, 9, 10, 11 -> "Good morning, "
                                        12, 13, 14, 15, 16, 17 -> "Good afternoon, "
                                        18, 19, 20, 21, 22, 23 -> "Good evening, "
                                        else -> "Evil day I guess, "
                                    },
                                    color = MaterialTheme.colorScheme.onSurface
                                )
                                Text(
                                    text = ((chatState.currentUser?.global_name ?: chatState.currentUser?.username) ?: "user"), // Lamp save us here
                                    fontWeight = FontWeight.Bold
                                )
                                Text(text = ".")
                            }
                        }   
                    }
                } else {
                    LoginScreen(onLoginSuccess = { })
                }
            } else {
                BoxWithConstraints(
                    modifier = Modifier
                        .fillMaxSize()
                        .onPreviewKeyEvent { event ->
                            if (event.type == KeyEventType.KeyDown) {
                                if (event.isCtrlPressed && event.key == Key.K) {
                                    chatState.isQuickSwitcherVisible = true
                                    return@onPreviewKeyEvent true
                                }
                            }
                            false
                        }
                ) {
                    val isCompact = maxWidth < 600.dp
                    if (isCompact) {
                        MobileBaseplate(chatState)
                    } else {
                        DesktopBaseplate(chatState)
                    }

                    if (chatState.isQuickSwitcherVisible) {
                        QuickSwitcher(
                            chatState = chatState,
                            onDismiss = { chatState.isQuickSwitcherVisible = false }
                        )
                    }

                    chatState.forwardingMessage?.let { message ->
                        ForwardDialog(
                            message = message,
                            chatState = chatState,
                            onDismiss = { chatState.forwardingMessage = null }
                        )
                    }
                }
            }
        }
    }
}
