package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.*

@Composable
actual fun MobileBaseplate(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    userStore: UserStore,
    voiceStore: VoiceStore
) {
    // Basic fallback for iOS
    Text("Mobile UI for iOS")
}
