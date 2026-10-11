package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalUriHandler
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.model.Sticker
import me.lampu.lampcord.shared.model.StickerPack
import me.lampu.lampcord.shared.state.EmojiStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import me.lampu.lampcord.shared.model.EmbedImage
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

// Smallest sticker tile; rows fit as many as the width allows and stretch them to fill it.
private val StickerMinSize = 80.dp

@Composable
fun StickerPicker(
    query: String = "",
    mediaApi: MediaApi,
    guildStore: GuildStore = koinInject(),
    emojiStore: EmojiStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    onStickerSelected: (Sticker) -> Unit
) {
    val guilds by guildStore.guilds.collectAsState()
    val selectedGuild = navigationStore.selectedGuild
    val uriHandler = LocalUriHandler.current
    var officialPacks by remember { mutableStateOf<List<StickerPack>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        isLoading = true
        officialPacks = mediaApi.getStickerPacks()?.sticker_packs ?: emptyList()
        isLoading = false
    }

    val stickerGroups = remember(guilds, officialPacks, selectedGuild, emojiStore.frequentStickers) {
        val groups = mutableListOf<StickerPack>()

        if (emojiStore.frequentStickers.isNotEmpty()) {
            val allStickers = guilds.flatMap { it.stickers } + officialPacks.flatMap { it.stickers }
            val frequent = emojiStore.frequentStickers.mapNotNull { id ->
                allStickers.find { it.id == id }
            }
            if (frequent.isNotEmpty()) {
                groups.add(StickerPack(
                    id = "frequent",
                    name = "Frequently Used",
                    stickers = frequent,
                    sku_id = ""
                ))
            }
        }

        selectedGuild?.let { guild ->
            if (guild.stickers.isNotEmpty()) {
                groups.add(StickerPack(
                    id = guild.id,
                    name = guild.name ?: "Current Server",
                    stickers = guild.stickers,
                    sku_id = ""
                ))
            }
        }

        guilds.filter { it.id != selectedGuild?.id && it.stickers.isNotEmpty() }.forEach { guild ->
            groups.add(StickerPack(
                id = guild.id,
                name = guild.name ?: "Unknown Server",
                stickers = guild.stickers,
                sku_id = ""
            ))
        }

        groups.addAll(officialPacks)

        groups
    }

    val filteredStickerGroups = remember(stickerGroups, query) {
        if (query.isBlank()) {
            stickerGroups
        } else {
            stickerGroups.mapNotNull { pack ->
                val matches = pack.stickers.filter { sticker ->
                    sticker.name.contains(query, ignoreCase = true) ||
                        sticker.description?.contains(query, ignoreCase = true) == true ||
                        sticker.tags?.contains(query, ignoreCase = true) == true
                }
                // A pack only stays when it still has something to show, otherwise its header
                // would sit above an empty strip.
                if (matches.isEmpty()) null else pack.copy(stickers = matches)
            }
        }
    }

    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()
    var cloneImageUrl by remember { mutableStateOf<String?>(null) }
    var showCloneModal by remember { mutableStateOf(false) }

    val groupOffsets = remember(filteredStickerGroups) {
        val offsets = mutableListOf<Int>()
        var acc = 0
        filteredStickerGroups.forEach { _ ->
            offsets.add(acc)
            acc += 2 // 1 for header, 1 for the sticker grid
        }
        offsets
    }

    var selectedGroupIndex by remember(filteredStickerGroups) { mutableStateOf(0) }

    fun groupForIndex(index: Int): Int {
        for (i in groupOffsets.indices) {
            val end = if (i + 1 < groupOffsets.size) groupOffsets[i + 1] else Int.MAX_VALUE
            if (index < end) return i
        }
        return 0
    }

    LaunchedEffect(listState) {
        snapshotFlow { listState.firstVisibleItemIndex }.collect { index: Int ->
            val group = groupForIndex(index)
            if (group != selectedGroupIndex) selectedGroupIndex = group
        }
    }

    if (isLoading && officialPacks.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            ContainedLoadingIndicator()
        }
    } else if (filteredStickerGroups.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                text = if (query.isBlank()) "No stickers available" else "No stickers found",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    } else {
        Column(modifier = Modifier.fillMaxSize()) {
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .weight(1f),
            contentPadding = PaddingValues(4.dp)
            ) {
                filteredStickerGroups.forEach { pack ->
                    item(key = "header_${pack.id}") {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.padding(top = 8.dp, bottom = 4.dp, start = 4.dp)
                        ) {
                            val guild = guilds.find { it.id == pack.id }
                            val iconUrl = when {
                                pack.id == "frequent" -> null
                                guild != null -> guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=64" }
                                else -> pack.cover_sticker_id?.let { "https://cdn.discordapp.com/stickers/$it.png?size=64" }
                            }

                            if (pack.id == "frequent") {
                                Icon(Icons.Filled.History, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                                Spacer(Modifier.width(8.dp))
                            } else if (iconUrl != null) {
                                AsyncImage(
                                    model = iconUrl,
                                    contentDescription = null,
                                    modifier = Modifier.size(20.dp).clip(CircleShape)
                                )
                                Spacer(Modifier.width(8.dp))
                            }

                            Text(
                                text = pack.name,
                                style = MaterialTheme.typography.labelSmall,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                    item(key = "stickers_${pack.id}") {
                        // Fixed 80dp tiles left the leftover width as an empty strip on the right. Fit as many
                        // columns as the width allows at that minimum size and stretch them to fill the row.
                        BoxWithConstraints(modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp)) {
                            val spacing = 8.dp
                            val columns = ((maxWidth + spacing) / (StickerMinSize + spacing)).toInt().coerceAtLeast(1)
                            Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                                pack.stickers.chunked(columns).forEach { rowStickers ->
                                    Row(horizontalArrangement = Arrangement.spacedBy(spacing)) {
                                        rowStickers.forEach { sticker ->
                                            val stickerUrl = "https://cdn.discordapp.com/stickers/${sticker.id}.png?size=160"
                                            val stickerMenu = listOf(
                                                ContextMenuItem("Copy Link", Icons.Filled.Link, onClick = {
                                                    setClipboardText(stickerUrl)
                                                    showToast("Copied to clipboard")
                                                }),
                                                ContextMenuItem("View Image", Icons.Filled.OpenInNew, onClick = {
                                                    val img = EmbedImage(url = stickerUrl, proxy_url = stickerUrl)
                                                    navigationStore.openAttachmentViewer(listOf(img))
                                                }),
                                                ContextMenuItem("Copy Sticker ID", Icons.Filled.Dns, onClick = {
                                                    setClipboardText(sticker.id)
                                                    showToast("Copied to clipboard")
                                                })
                                            )

                                            val extra = listOf(
                                                ContextMenuItem("Save Image", Icons.Filled.Download, onClick = {
                                                    uriHandler.openUri(stickerUrl)
                                                }),
                                                ContextMenuItem("Clone to other server", Icons.Filled.Upload, onClick = {
                                                    cloneImageUrl = stickerUrl
                                                    showCloneModal = true
                                                })
                                            )

                                            ContextMenu(items = stickerMenu + extra, modifier = Modifier.weight(1f).aspectRatio(1f)) {
                                                AsyncImage(
                                                    model = stickerUrl,
                                                    contentDescription = sticker.name,
                                                    modifier = Modifier
                                                        .fillMaxSize()
                                                        .clickableCursor {
                                                            emojiStore.onStickerUsed(sticker.id)
                                                            onStickerSelected(sticker)
                                                        }
                                                        .padding(4.dp)
                                                )
                                            }
                                        }
                                        // Keep a short last row on the same grid instead of stretching its tiles.
                                        repeat(columns - rowStickers.size) { Spacer(Modifier.weight(1f)) }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if (filteredStickerGroups.size > 1) {
                StickerServerBar(
                    groups = filteredStickerGroups,
                    selectedIndex = selectedGroupIndex,
                    onSelect = { index ->
                        selectedGroupIndex = index
                        coroutineScope.launch {
                            if (!Settings.shared.reduceMotion) listState.animateScrollToItem(groupOffsets[index]) else listState.scrollToItem(groupOffsets[index])
                        }
                    }
                )
            }
            if (showCloneModal && cloneImageUrl != null) {
                CloneToServerModal(imageUrl = cloneImageUrl!!, defaultName = "sticker", onDismiss = { showCloneModal = false })
            }
        }
    }
}

@Composable
private fun StickerServerBar(
    groups: List<StickerPack>,
    selectedIndex: Int,
    guildStore: GuildStore = koinInject(),
    onSelect: (Int) -> Unit
) {
    val guilds by guildStore.guilds.collectAsState()
    Column {
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        LazyRow(
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            itemsIndexed(groups) { index, group ->
                val selected = selectedIndex == index
                val guild = guilds.find { it.id == group.id }
                val iconUrl = if (guild != null) {
                    guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=64" }
                } else {
                    group.cover_sticker_id?.let { "https://cdn.discordapp.com/stickers/$it.png?size=64" }
                }

                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Box(
                        modifier = Modifier
                            .width(36.dp)
                            .height(3.dp)
                            .clip(RoundedCornerShape(1.5.dp))
                            .background(if (selected) MaterialTheme.colorScheme.primary else Color.Transparent)
                    )
                    Box(
                        modifier = Modifier
                            .padding(top = 4.dp, bottom = 4.dp)
                            .size(36.dp)
                            .clip(CircleShape)
                            .background(if (selected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant)
                            .clickableCursor { onSelect(index) },
                        contentAlignment = Alignment.Center
                    ) {
                        if (group.id == "frequent") {
                            Icon(
                                imageVector = Icons.Filled.History,
                                contentDescription = "Frequently Used",
                                modifier = Modifier.size(20.dp),
                                tint = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        } else if (iconUrl != null) {
                            AsyncImage(
                                model = iconUrl,
                                contentDescription = group.name,
                                modifier = Modifier.fillMaxSize()
                            )
                        } else {
                            Icon(
                                imageVector = Icons.Filled.Group,
                                contentDescription = group.name,
                                modifier = Modifier.size(20.dp),
                                tint = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }
}
