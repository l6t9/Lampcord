package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull

@Composable
fun GuildRail(chatState: ChatState, modifier: Modifier = Modifier) {
    val railScrollState = rememberLazyListState()
    LazyColumn(
        state = railScrollState,
        modifier = modifier
            .width(72.dp)
            .fillMaxHeight(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
        contentPadding = PaddingValues(vertical = 8.dp)
    ) {
        item {
            val isHomeSelected = chatState.selectedGuild == null
            RegularGuildItem(
                isSelected = isHomeSelected,
                onClick = { chatState.selectHome() },
                selectedColor = MaterialTheme.colorScheme.primary,
                unselectedColor = MaterialTheme.colorScheme.surfaceVariant
            ) {
                Icon(
                    imageVector = Icons.Brand.Discord,
                    contentDescription = "Home",
                    tint = if (isHomeSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(35.dp)
                )
            }
        }

        item {
            HorizontalDivider(
                modifier = Modifier
                    .width(32.dp)
                    .padding(vertical = 4.dp)
                    .clip(MaterialTheme.shapes.medium),
                thickness = 2.dp,
                color = MaterialTheme.colorScheme.outline,
            )
        }

        val folders = chatState.userSettings?.guild_folders ?: emptyList()

        if (folders.isEmpty()) {
            items(chatState.guilds.distinctBy { it.id }, key = { it.id }) { guild ->
                GuildIcon(
                    guild = guild,
                    isSelected = chatState.selectedGuild?.id == guild.id,
                    chatState = chatState,
                    onClick = { chatState.selectGuild(guild) }
                )
            }
        } else {
            items(folders) { folder ->
                val guildIds = folder.guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() }
                if (folder.id == null && guildIds.size == 1) {
                    val guildId = guildIds.first()
                    val guild = chatState.guilds.find { it.id == guildId }
                    if (guild != null) {
                        GuildIcon(
                            guild = guild,
                            isSelected = chatState.selectedGuild?.id == guild.id,
                            chatState = chatState,
                            onClick = { chatState.selectGuild(guild) }
                        )
                    }
                } else {
                    GuildFolderItem(folder, chatState)
                }
            }
        }
    }
}
