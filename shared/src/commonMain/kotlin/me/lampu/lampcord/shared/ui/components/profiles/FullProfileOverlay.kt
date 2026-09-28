package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.state.ProfileStore

/**
 * Hosts the full profile view, which is a desktop-only affordance.
 *
 * Mobile keeps the compact popup and opens the avatar in the attachment viewer instead, so the
 * dialog itself only ships in the desktop source set.
 */
@Composable
expect fun FullProfileOverlay(profileStore: ProfileStore)
