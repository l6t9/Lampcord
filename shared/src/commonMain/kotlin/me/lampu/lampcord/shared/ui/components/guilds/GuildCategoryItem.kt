package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.ChannelItem
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject

@Composable
fun GuildCategoryItem(
    category: Channel,
    channels: List<Channel>,
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    guildStore: GuildStore = koinInject()
) {
    var collapsed by remember { mutableStateOf(false) }
    val guild = navigationStore.selectedGuild
    val currentUser by userStore.currentUser.collectAsState()
    val allMembers by userStore.members.collectAsState()
    
    val member = remember(guild?.id, currentUser, allMembers) {
        val g = guild
        val u = currentUser
        if (g != null && u != null) allMembers[g.id]?.get(u.id) else null
    }
    val showHidden = settingsStore.showHiddenChannels

    val currentUserId = currentUser?.id
    val categoryChannels = remember(channels, category.id, guild, member, showHidden, currentUserId) {
        val g = guild
        channels.filter { channel -> 
            channel.parent_id == category.id && (if (member == null || showHidden || g == null) true else PermissionHelper.canViewChannel(member, g, channel, currentUserId))
        }.sortedBy { it.position ?: 0 }
    }

    if (categoryChannels.isEmpty() && !showHidden) return

    val categoryContextMenuItems = remember(category, settingsStore.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Mark As Read", Icons.Filled.Check) {
                guildStore.markCategoryAsRead(category.id)
            }
        )
        if (settingsStore.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns) { setClipboardText(category.id) })
        }
        items
    }

    Column {
        ContextMenu(items = categoryContextMenuItems) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { collapsed = !collapsed }
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = me.lampu.lampcord.shared.utils.CleanUtils.cleanChannelName(category.name ?: "Category", isCategory = true),
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(modifier = Modifier.width(4.dp))
                Icon(
                    imageVector = if (collapsed) Icons.Filled.ChevronRight else Icons.Filled.KeyboardArrowDown,
                    contentDescription = if (collapsed) "Expand" else "Collapse",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(16.dp)
                )
            }
        }
        if (!collapsed) {
            val sortedCategoryChannels =
                channels.filter { it.parent_id == category.id }
                    .distinctBy { it.id }
                    .sortedWith(compareBy({ it.type == 2 || it.type == 13 }, { it.position ?: 0 }))
            Column {
                sortedCategoryChannels.forEach { channel ->
                    ChannelItem(channel)
                }
            }
        }
    }
}
