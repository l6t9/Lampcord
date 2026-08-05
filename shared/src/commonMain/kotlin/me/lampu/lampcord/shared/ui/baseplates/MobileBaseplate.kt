package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.ChatState

@Composable
expect fun MobileBaseplate(chatState: ChatState)
