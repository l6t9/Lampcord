package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.ChatState

@Composable
actual fun MobileBaseplate(chatState: ChatState) {
    // Basic fallback for iOS
    Text("Mobile UI for iOS")
}
