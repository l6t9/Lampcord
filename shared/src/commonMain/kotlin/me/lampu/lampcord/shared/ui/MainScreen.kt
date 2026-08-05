package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.input.key.*
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.baseplates.DesktopBaseplate
import me.lampu.lampcord.shared.ui.baseplates.MobileBaseplate
import me.lampu.lampcord.shared.ui.components.*
import org.koin.compose.koinInject

@Composable
fun MainScreen(
    chatState: ChatState
) {
    if (!chatState.isConnected) {
        if (chatState.isConnecting) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                ContainedLoadingIndicator()
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
