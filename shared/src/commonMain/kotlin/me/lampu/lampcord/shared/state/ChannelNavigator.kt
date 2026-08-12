package me.lampu.lampcord.shared.state

import androidx.compose.runtime.snapshotFlow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import me.lampu.lampcord.shared.gateway.GatewayManager

class ChannelNavigator(
    private val navigationStore: NavigationStore,
    private val entityStore: EntityStore,
    private val gatewayManager: GatewayManager,
    private val scope: CoroutineScope
) {
    fun navigateToChannel(channelId: String, guildId: String? = null) {
        scope.launch {
            if (!navigationStore.isConnected) {
                withTimeoutOrNull(20_000) {
                    snapshotFlow { navigationStore.isConnected }.first { it }
                }
            }
            if (!navigationStore.isConnected) return@launch

            navigationStore.isSettingsVisible = false
            navigationStore.isQuickSwitcherVisible = false
            navigationStore.isSearchVisible = false
            navigationStore.isPinsVisible = false
            navigationStore.isServerMenuVisible = false

            val channel = entityStore.channels.value[channelId]
            if (channel == null) {
                val guild = guildId?.let { entityStore.guilds.value[it] }
                if (guild != null) {
                    navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) }
                }
                return@launch
            }

            if (channel.guild_id != null) {
                val guild = entityStore.guilds.value[channel.guild_id] ?: return@launch
                if (navigationStore.selectedGuild?.id == guild.id) {
                    navigationStore.selectChannel(channel)
                } else {
                    navigationStore.selectGuild(guild, channelId) { gatewayManager.sendSubscription(it) }
                }
            } else {
                navigationStore.selectedGuild = null
                navigationStore.isFriendsSelected = false
                navigationStore.selectChannel(channel)
            }
        }
    }
}
