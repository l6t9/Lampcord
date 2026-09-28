package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.ProfileStore

@Composable
actual fun FullProfileOverlay(profileStore: ProfileStore) {
    val profile = profileStore.selectedProfile ?: return
    if (!profileStore.isFullProfileVisible) return

    FullProfileDialog(
        profile = profile,
        onDismiss = { profileStore.closeFullProfile() }
    )
}
