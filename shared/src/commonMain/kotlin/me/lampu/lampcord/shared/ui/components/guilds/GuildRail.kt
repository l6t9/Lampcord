package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.components.ExpressiveTooltip
import me.lampu.lampcord.shared.ui.components.tooltipText
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.state.ReadStateStore
import androidx.compose.runtime.remember
import androidx.compose.runtime.derivedStateOf

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.graphics.Color
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.theme.*

@Composable
fun GuildRail(
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    readStateStore: ReadStateStore = koinInject(),
    userStore: UserStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    userGuildSettingsStore: me.lampu.lampcord.shared.state.UserGuildSettingsStore = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    modifier: Modifier = Modifier
) {
    val railScrollState = rememberLazyListState()
    val guilds by guildStore.guilds.collectAsState()
    val privateChannels by guildStore.privateChannels.collectAsState()
    val readStates by readStateStore.readStates.collectAsState()
    val userSettings = settingsStore.userSettings

    val dmMentionChannels by remember(privateChannels, readStates, navigationStore.selectedChannel) {
        derivedStateOf {
            privateChannels.filter { 
                readStateStore.getMentionCount(it.id) > 0 && 
                        it.id != navigationStore.selectedChannel?.id &&
                        !userGuildSettingsStore.isChannelMuted(null, it.id)
            }
        }
    }

    LazyColumn(
        state = railScrollState,
        modifier = modifier
            .width(72.dp)
            .fillMaxHeight(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
        contentPadding = PaddingValues(vertical = 8.dp)
    ) {
        item {
            val isHomeSelected = navigationStore.selectedGuild == null
            
            val totalDmMentions by remember(privateChannels, readStates) {
                derivedStateOf {
                    privateChannels.sumOf { readStateStore.getMentionCount(it.id) }
                }
            }

            ExpressiveTooltip(
                anchorPosition = TooltipAnchorPosition.End,
                content = tooltipText("Direct Messages"),
                anchor = {
                    Box(contentAlignment = Alignment.Center) {
                        RegularGuildItem(
                            isSelected = isHomeSelected,
                            onClick = { navigationStore.selectHome() },
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

                        if (totalDmMentions > 0) {
                            Box(modifier = Modifier.size(48.dp)) {
                                Surface(
                                    color = MaterialTheme.colorScheme.error,
                                    shape = CircleShape,
                                    modifier = Modifier
                                        .align(Alignment.BottomEnd)
                                        .offset(x = 3.dp, y = 3.dp)
                                        .height(20.dp)
                                        .widthIn(min = 20.dp),
                                    shadowElevation = 2.dp,
                                    border = if (isHomeSelected) null else androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.surface)
                                ) {
                                    Box(
                                        contentAlignment = Alignment.Center,
                                        modifier = Modifier.padding(horizontal = 5.dp)
                                    ) {
                                        Text(
                                            text = if (totalDmMentions > 99) "99+" else totalDmMentions.toString(),
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onError,
                                            fontWeight = FontWeight.ExtraBold,
                                            fontSize = 11.sp,
                                            maxLines = 1
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            )
        }

        items(dmMentionChannels, key = { "dm_${it.id}" }) { channel ->
            DMIcon(channel = channel)
        }

        item {
            HorizontalDivider(
                modifier = Modifier
                    .width(32.dp)
                    .padding(vertical = 2.dp)
                    .clip(MaterialTheme.shapes.medium),
                thickness = 2.dp,
                color = MaterialTheme.colorScheme.outline,
            )
        }

        val folders = userSettings?.guild_folders ?: emptyList()

        if (folders.isEmpty()) {
            items(guilds.distinctBy { it.id }, key = { it.id }) { guild ->
                GuildIcon(
                    guild = guild,
                    isSelected = navigationStore.selectedGuild?.id == guild.id,
                    onClick = { navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) } }
                )
            }
        } else {
            items(folders) { folder ->
                val guildIds = folder.guild_ids.map { it.jsonPrimitive.contentOrNull ?: it.toString() }
                if (folder.id == null && guildIds.size == 1) {
                    val guildId = guildIds.first()
                    val guild = guilds.find { it.id == guildId }
                    if (guild != null) {
                        GuildIcon(
                            guild = guild,
                            isSelected = navigationStore.selectedGuild?.id == guild.id,
                            onClick = { navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) } }
                        )
                    }
                } else {
                    GuildFolderItem(folder)
                }
            }
        }
    }
}

@Composable
private fun DMIcon(
    channel: Channel,
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject(),
    readStateStore: ReadStateStore = koinInject()
) {
    val isSelected = navigationStore.selectedChannel?.id == channel.id && navigationStore.selectedGuild == null
    val allUsers by userStore.users.collectAsState()
    val readStates by readStateStore.readStates.collectAsState()
    
    val recipient = remember(channel, allUsers) {
        val recipientId = channel.recipients?.firstOrNull()?.id 
            ?: channel.recipient_ids?.firstOrNull()
            ?: return@remember null
        allUsers[recipientId] ?: channel.recipients?.firstOrNull()
    }

    val iconUrl = if (channel.type == 3) {
        if (channel.icon != null) "https://cdn.discordapp.com/channel-icons/${channel.id}/${channel.icon}.png?size=96"
        else null
    } else recipient?.let { CdnUrls.getUserAvatarUrl(it.id, it.avatar, 96) }

    val mentionCount by remember(channel.id, readStates) {
        derivedStateOf { readStateStore.getMentionCount(channel.id) }
    }
    
    val isUnread by remember(channel, readStates) {
        derivedStateOf { readStateStore.isUnread(channel) }
    }

    ExpressiveTooltip(
        anchorPosition = TooltipAnchorPosition.End,
        content = tooltipText(channel.name ?: recipient?.global_name ?: recipient?.username ?: "Direct Message"),
        anchor = {
            Box(contentAlignment = Alignment.Center) {
                RegularGuildItem(
                    isSelected = isSelected,
                    isUnread = isUnread,
                    onClick = { 
                        navigationStore.selectedGuild = null
                        navigationStore.selectChannel(channel, explicitlySelected = true) 
                    },
                    selectedColor = if (iconUrl == null) MaterialTheme.colorScheme.primary else Color.Transparent,
                    unselectedColor = if (iconUrl == null) MaterialTheme.colorScheme.surfaceVariant else Color.Transparent
                ) {
                    if (iconUrl != null) {
                        AsyncImage(
                            model = iconUrl,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize()
                        )
                    } else {
                        val initials = (channel.name ?: recipient?.global_name ?: recipient?.username ?: "?").take(1)
                        Text(
                            text = initials,
                            style = MaterialTheme.typography.titleMedium,
                            color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary
                        )
                    }
                }

                if (mentionCount > 0) {
                    Box(modifier = Modifier.size(48.dp)) {
                        Surface(
                            color = MaterialTheme.colorScheme.error,
                            shape = CircleShape,
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .offset(x = 3.dp, y = 3.dp)
                                .height(20.dp)
                                .widthIn(min = 20.dp),
                            shadowElevation = 2.dp,
                            border = if (isSelected) null else androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.surface)
                        ) {
                            Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(horizontal = 5.dp)) {
                                Text(
                                    text = if (mentionCount > 99) "99+" else mentionCount.toString(),
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onError,
                                    fontWeight = FontWeight.ExtraBold,
                                    fontSize = 11.sp,
                                    maxLines = 1
                                )
                            }
                        }
                    }
                }
            }
        }
    )
}
