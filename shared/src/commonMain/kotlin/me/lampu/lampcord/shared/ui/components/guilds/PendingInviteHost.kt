package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import me.lampu.lampcord.shared.state.NavigationStore
import org.koin.compose.koinInject

@Composable
fun PendingInviteHost(navigationStore: NavigationStore = koinInject()) {
    val invite = navigationStore.pendingInvite ?: return
    val guildId = invite.guild?.id ?: return
    key(invite.code, guildId) {
        GuildProfileSheet(
            guildId = guildId,
            invite = invite,
            onDismiss = { navigationStore.pendingInvite = null }
        )
    }
}
