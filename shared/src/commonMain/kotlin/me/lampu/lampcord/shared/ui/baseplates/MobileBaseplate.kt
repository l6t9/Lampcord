package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.VoiceStore
import org.koin.compose.koinInject

@Composable
expect fun MobileBaseplate(
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    userStore: UserStore = koinInject(),
    voiceStore: VoiceStore = koinInject()
)
