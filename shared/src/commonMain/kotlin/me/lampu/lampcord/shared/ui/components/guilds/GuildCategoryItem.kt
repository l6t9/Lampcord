package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ChannelItem
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun GuildCategoryItem(
    category: Channel,
    channels: List<Channel>,
    chatState: ChatState
) {
    var collapsed by remember { mutableStateOf(false) }
    val guild = chatState.selectedGuild
    val member = chatState.currentMember
    val showHidden = chatState.settingsStore.showHiddenChannels

    val categoryChannels = remember(channels, category.id, guild, member, showHidden) {
        channels.filter { it.parent_id == category.id }.sortedBy { it.position ?: 0 }
    }

    if (categoryChannels.isEmpty() && !showHidden) return

    val categoryContextMenuItems = remember(category, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Mark As Read", Icons.Filled.Check) {
                chatState.markCategoryAsRead(category.id)
            }
        )
        if (chatState.userSettings?.developer_mode == true) {
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
                    ChannelItem(channel, chatState)
                }
            }
        }
    }
}
