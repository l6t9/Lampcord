package com.example.lampcord.shared.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.baseplates.DesktopBaseplate
import com.example.lampcord.shared.ui.baseplates.MobileBaseplate
import com.example.lampcord.shared.ui.components.*
import org.koin.compose.koinInject

@Composable
fun MainScreen(
    chatState: ChatState = koinInject()
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
        BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
            val isCompact = maxWidth < 600.dp
            if (isCompact) {
                MobileBaseplate(chatState)
            } else {
                DesktopBaseplate(chatState)
            }
        }
    }
}
