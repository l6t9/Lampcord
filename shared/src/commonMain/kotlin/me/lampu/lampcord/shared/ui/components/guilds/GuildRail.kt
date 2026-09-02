package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
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
import androidx.compose.ui.graphics.graphicsLayer
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
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.GuildFolder
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.theme.*
import kotlinx.serialization.json.JsonPrimitive
import kotlin.math.abs
import me.lampu.lampcord.shared.utils.getPlatformName

private data class GuildRailEntry(
    val key: String,
    val guild: Guild? = null,
    val folder: GuildFolder? = null
)

private fun GuildFolder.guildIds(): List<String> = guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull }

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

    val haptic = LocalHapticFeedback.current
    val folders = userSettings?.guild_folders ?: emptyList()
    val canArrange = getPlatformName() == "android"
    var arrangeMode by remember { mutableStateOf(false) }
    var draggingKey by remember { mutableStateOf<String?>(null) }
    var dragOffsetPx by remember { mutableFloatStateOf(0f) }

    val railEntries = remember(guilds, folders) {
        val guildsById = guilds.distinctBy { it.id }.associateBy { it.id }
        // Discord's guild_folders array is already the complete sidebar
        // sequence, including anonymous single-guild entries.
        val entries = if (folders.isNotEmpty()) {
            folders.mapIndexedNotNull { index, folder ->
                val guildIds = folder.guildIds()
                if (folder.id == null && guildIds.size == 1) {
                    guildsById[guildIds.first()]?.let { guild ->
                        GuildRailEntry(key = "guild:${guild.id}", guild = guild)
                    }
                } else {
                    GuildRailEntry(key = folder.railKey(index), folder = folder)
                }
            }.toMutableList()
        } else {
            guilds.map { it.id }.distinct().mapNotNull { guildsById[it] }
                .map { guild -> GuildRailEntry(key = "guild:${guild.id}", guild = guild) }
                .toMutableList()
        }

        // Match Discord's ensureValidPositions behavior: a newly arrived guild
        // missing from the snapshot is temporarily inserted at the top.
        val representedGuildIds = folders.flatMap { it.guildIds() }.toSet()
        val missingGuilds = guilds.filter { guild ->
            guild.id !in representedGuildIds && entries.none { it.guild?.id == guild.id }
        }
        missingGuilds.asReversed().forEach { guild ->
            entries.add(0, GuildRailEntry(key = "guild:${guild.id}", guild = guild))
        }
        entries
    }

    fun finishDrag() {
        val sourceKey = draggingKey ?: return
        val sourceInfo = railScrollState.layoutInfo.visibleItemsInfo.firstOrNull { it.key == sourceKey }
        val targetKey = sourceInfo?.let { info ->
            val draggedCenter = info.offset + dragOffsetPx + info.size / 2
            railScrollState.layoutInfo.visibleItemsInfo
                .filter { it.key != sourceKey && (it.key.toString().startsWith("guild:") || it.key.toString().startsWith("folder:")) }
                .minByOrNull { abs(it.offset + it.size / 2 - draggedCenter) }
                ?.key as? String
        }
        if (canArrange && targetKey != null && targetKey != sourceKey) {
            val partial = UserSettings.Partial(guild_folders = rearrangeGuildFolders(railEntries, sourceKey, targetKey))
            settingsStore.handlePartialUpdate(partial)
            settingsStore.updateUserSettings(partial)
        }
        draggingKey = null
        dragOffsetPx = 0f
        arrangeMode = false
    }

    fun cancelDrag() {
        draggingKey = null
        dragOffsetPx = 0f
    }

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
                    Box(contentAlignment = Alignment.Center, modifier = Modifier.graphicsLayer(clip = false)) {
                        RegularGuildItem(
                            isSelected = isHomeSelected,
                            isMonogram = true,
                            onClick = {
                                navigationStore.selectHome()
                                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                            },
                            selectedColor = MaterialTheme.colorScheme.primary,
                            unselectedColor = MaterialTheme.colorScheme.surfaceVariant,
                            monogramSelectedColor = MaterialTheme.colorScheme.onPrimary,
                            monogramUnselectedColor = MaterialTheme.colorScheme.primary
                        ) {
                            Icon(
                                imageVector = Icons.Brand.Discord,
                                contentDescription = "Home",
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

        items(railEntries, key = { it.key }) { entry ->
            val folder = entry.folder
            val guild = entry.guild
            if (guild != null) {
                GuildIcon(
                    guild = guild,
                    isSelected = navigationStore.selectedGuild?.id == guild.id,
                    arrangeMode = canArrange,
                    isDragging = draggingKey == entry.key,
                    dragOffset = dragOffsetPx,
                    onArrangeMode = { arrangeMode = true },
                    onDragStart = { draggingKey = entry.key; dragOffsetPx = 0f },
                    onDrag = { dragOffsetPx += it },
                    onDragEnd = ::finishDrag,
                    onDragCancel = ::cancelDrag,
                    onClick = {
                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                        navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) }
                    }
                )
            } else if (folder != null) {
                val guildIds = folder.guildIds()
                if (folder.id == null && guildIds.size == 1) {
                    val singletonGuild = guilds.firstOrNull { it.id == guildIds.first() }
                    if (singletonGuild != null) {
                        GuildIcon(
                            guild = singletonGuild,
                            isSelected = navigationStore.selectedGuild?.id == singletonGuild.id,
                            onClick = {
                                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                navigationStore.selectGuild(singletonGuild) { gatewayManager.sendSubscription(it) }
                            }
                        )
                    }
                } else {
                    GuildFolderItem(
                        folder = folder,
                        arrangeMode = canArrange,
                        onArrangeMode = { arrangeMode = true },
                        onDragStart = { draggingKey = entry.key; dragOffsetPx = 0f },
                        onDrag = { dragOffsetPx += it },
                        onDragEnd = ::finishDrag,
                        onDragCancel = ::cancelDrag
                        , isDragging = draggingKey == entry.key
                        , dragOffset = dragOffsetPx
                    )
                }
            }
        }
    }
}

private fun rearrangeGuildFolders(
    entries: List<GuildRailEntry>,
    sourceKey: String,
    targetKey: String
): List<GuildFolder> {
    val current = entries.map { entry ->
        entry.folder ?: GuildFolder(guild_ids = listOf(JsonPrimitive(entry.guild!!.id)))
    }.toMutableList()
    val sourceIndex = entries.indexOfFirst { it.key == sourceKey }
    val targetIndex = entries.indexOfFirst { it.key == targetKey }
    if (sourceIndex < 0 || targetIndex < 0) return current

    val sourceFolder = entries[sourceIndex].folder
    if (sourceFolder != null) {
        val moved = current.removeAt(sourceIndex)
        current.add((if (sourceIndex < targetIndex) targetIndex - 1 else targetIndex).coerceIn(0, current.size), moved)
        return current
    }

    val guildId = entries[sourceIndex].guild?.id ?: return current
    current.indices.forEach { index ->
        current[index] = current[index].copy(
            guild_ids = current[index].guild_ids.filterNot { it.jsonPrimitive.contentOrNull == guildId }
        )
    }
    current.removeAll { it.guild_ids.isEmpty() }

    val targetFolder = entries[targetIndex].folder
    if (targetFolder != null) {
        val actualTarget = current.indexOfFirst { it.id == targetFolder.id }
        if (actualTarget >= 0) {
            current[actualTarget] = current[actualTarget].copy(
                guild_ids = current[actualTarget].guild_ids + JsonPrimitive(guildId)
            )
        }
    } else {
        val targetGuildId = entries[targetIndex].guild?.id
        val insertionIndex = current.indexOfFirst { folder ->
            folder.id == null && folder.guild_ids.any { it.jsonPrimitive.contentOrNull == targetGuildId }
        }.takeIf { it >= 0 } ?: current.size
        current.add(insertionIndex, GuildFolder(guild_ids = listOf(JsonPrimitive(guildId))))
    }
    return current.filter { it.guild_ids.isNotEmpty() }
}

private fun GuildFolder.railKey(index: Int): String = id?.let { "folder:$it" } ?: "folder:$index:${guildIds().joinToString(",")}"

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
            Box(contentAlignment = Alignment.Center, modifier = Modifier.graphicsLayer(clip = false)) {
                RegularGuildItem(
                    isSelected = isSelected,
                    isUnread = isUnread,
                    isMonogram = (iconUrl == null),
                    onClick = {
                        navigationStore.selectedGuild = null
                        navigationStore.selectChannel(channel, explicitlySelected = true) 
                    },
                    selectedColor = if (iconUrl == null) MaterialTheme.colorScheme.primary else Color.Transparent,
                    unselectedColor = if (iconUrl == null) MaterialTheme.colorScheme.surfaceVariant else Color.Transparent,
                    monogramSelectedColor = MaterialTheme.colorScheme.onPrimary,
                    monogramUnselectedColor = MaterialTheme.colorScheme.primary
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
                            style = MaterialTheme.typography.titleMedium
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
