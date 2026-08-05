package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.ChatState

@Composable
actual fun MobileBaseplate(chatState: ChatState) {
    // Desktop usually uses DesktopBaseplate, but satisfying actual requirement
    Text("Mobile UI not optimized for Desktop")
}
