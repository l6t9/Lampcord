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
import me.lampu.lampcord.shared.utils.downloadToDownloads

@Composable
fun StickerPicker(
    mediaApi: MediaApi,
    guildStore: GuildStore = koinInject(),
    emojiStore: EmojiStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    onStickerSelected: (Sticker) -> Unit
) {
    val guilds by guildStore.guilds.collectAsState()
    val selectedGuild = navigationStore.selectedGuild
    var officialPacks by remember { mutableStateOf<List<StickerPack>>(emptyList()) }
    var isLoading by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        isLoading = true
        officialPacks = mediaApi.getStickerPacks()?.sticker_packs ?: emptyList()
        isLoading = false
    }

    val stickerGroups = remember(guilds, officialPacks, selectedGuild, emojiStore.frequentStickers) {
        val groups = mutableListOf<StickerPack>()

        // 1. Frequently Used
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

        // 2. Current Server
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

        // 3. Other Guilds
        guilds.filter { it.id != selectedGuild?.id && it.stickers.isNotEmpty() }.forEach { guild ->
            groups.add(StickerPack(
                id = guild.id,
                name = guild.name ?: "Unknown Server",
                stickers = guild.stickers,
                sku_id = ""
            ))
        }

        // 4. Official Packs
        groups.addAll(officialPacks)

        groups
    }

    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()
    var cloneImageUrl by remember { mutableStateOf<String?>(null) }
    var showCloneModal by remember { mutableStateOf(false) }

    val groupOffsets = remember(stickerGroups) {
        val offsets = mutableListOf<Int>()
        var acc = 0
        stickerGroups.forEach { _ ->
            offsets.add(acc)
            acc += 2 // 1 for header, 1 for stickers (FlowRow)
        }
        offsets
    }

    var selectedGroupIndex by remember(stickerGroups) { mutableStateOf(0) }

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
    } else {
        Column(modifier = Modifier.fillMaxSize()) {
            LazyColumn(
                state = listState,
                modifier = Modifier.weight(1f),
                contentPadding = PaddingValues(4.dp)
            ) {
                stickerGroups.forEach { pack ->
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
                        FlowRow(
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            pack.stickers.forEach { sticker ->
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

                                // Save and Clone
                                val downloadFilename = "sticker_${sticker.id}.png"
                                val extra = listOf(
                                    ContextMenuItem("Save Image", Icons.Filled.Download, onClick = {
                                        val scope = coroutineScope
                                        scope.launch {
                                            val ok = downloadToDownloads(stickerUrl, downloadFilename)
                                            if (ok) showToast("Saved to Downloads") else showToast("Save failed")
                                        }
                                    }),
                                    ContextMenuItem("Clone to other server", Icons.Filled.Upload, onClick = {
                                        cloneImageUrl = stickerUrl
                                        showCloneModal = true
                                    })
                                )

                                ContextMenu(items = stickerMenu + extra) {
                                    AsyncImage(
                                        model = stickerUrl,
                                        contentDescription = sticker.name,
                                        modifier = Modifier
                                            .size(80.dp)
                                            .clickable {
                                                emojiStore.onStickerUsed(sticker.id)
                                                onStickerSelected(sticker)
                                            }
                                            .padding(4.dp)
                                    )
                                }
                            }
                        }
                    }
                }
            }

            if (stickerGroups.size > 1) {
                StickerServerBar(
                    groups = stickerGroups,
                    selectedIndex = selectedGroupIndex,
                    onSelect = { index ->
                        selectedGroupIndex = index
                        coroutineScope.launch {
                            if (Settings.shared.reduceMotion) listState.scrollToItem(groupOffsets[index]) else listState.animateScrollToItem(groupOffsets[index])
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
                            .clickable { onSelect(index) },
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
