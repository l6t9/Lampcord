@file:OptIn(ExperimentalMaterial3Api::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.*
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun EmojiPicker(
    userStore: UserStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    modifier: Modifier = Modifier,
    onEmojiSelected: (Emoji) -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    val currentUser by userStore.currentUser.collectAsState()
    val nitro = (currentUser?.premium_type ?: 0) > 0 || me.lampu.lampcord.shared.settings.Settings.shared.freeNitroEmojis
    
    var defaultEmojis by remember { mutableStateOf<List<Emoji>>(emptyList()) }
    LaunchedEffect(Unit) {
        defaultEmojis = EmojiLoader.getDefaultEmojis()
    }

    val guilds by guildStore.guilds.collectAsState()
    val selectedGuild = navigationStore.selectedGuild
    val emojiGroups = remember(selectedGuild, guilds.size, nitro, defaultEmojis) {
        val groups = mutableListOf<EmojiGroup>()
        
        if (nitro) {
            groups.addAll(
                guilds
                    .sortedByDescending { it.id == selectedGuild?.id }
                    .map { guild ->
                        EmojiGroup(guild.id, guild.name, guild.emojis, guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=64" })
                    }.filter { it.emojis.isNotEmpty() }
            )
        } else {
            selectedGuild?.let { guild ->
                groups.add(EmojiGroup(guild.id, guild.name, guild.emojis, guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=64" }))
            }
        }
        
        if (defaultEmojis.isNotEmpty()) {
            groups.add(EmojiGroup("standard", "Standard Emojis", defaultEmojis, null))
        }
        
        groups
    }

    val gridState = rememberLazyGridState()
    val coroutineScope = rememberCoroutineScope()
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"

    var selectedGroupIndex by remember(emojiGroups) { mutableStateOf(0) }
    val groupOffsets = remember(emojiGroups) {
        val offsets = mutableListOf<Int>()
        var acc = 0
        emojiGroups.forEach { group ->
            offsets.add(acc)
            acc += 1 + group.emojis.size
        }
        offsets
    }
    fun groupForIndex(index: Int): Int {
        for (i in groupOffsets.indices) {
            val end = if (i + 1 < groupOffsets.size) groupOffsets[i + 1] else Int.MAX_VALUE
            if (index < end) return i
        }
        return 0
    }

    LaunchedEffect(gridState, groupOffsets) {
        snapshotFlow { gridState.firstVisibleItemIndex }.collect { index ->
            val group = groupForIndex(index)
            if (group != selectedGroupIndex) selectedGroupIndex = group
        }
    }

    Surface(
        modifier = if (isMobile) modifier.fillMaxWidth().height(400.dp) else modifier.width(400.dp).height(500.dp),
        shape = if (isMobile) RoundedCornerShape(0.dp) else RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shadowElevation = if (isMobile) 0.dp else 8.dp
    ) {
        Row(modifier = Modifier.fillMaxSize()) {
            Column(modifier = Modifier.weight(1f)) {
                SecondaryTabRow(
                    selectedTabIndex = selectedTab,
                    containerColor = Color.Transparent,
                    contentColor = MaterialTheme.colorScheme.primary,
                    divider = {}
                ) {
                    Tab(selected = selectedTab == 0, onClick = { selectedTab = 0 }) {
                        Box(Modifier.padding(12.dp)) {
                            Icon(Icons.Filled.AddReaction, "Emojis")
                        }
                    }
                    Tab(selected = selectedTab == 1, onClick = { selectedTab = 1 }) {
                        Box(Modifier.padding(12.dp)) {
                            Icon(Icons.Filled.Gif, "GIFs")
                        }
                    }
                    Tab(selected = selectedTab == 2, onClick = { selectedTab = 2 }) {
                        Box(Modifier.padding(12.dp)) {
                            Icon(Icons.Filled.StickyNote2, "Stickers")
                        }
                    }
                }

                Box(modifier = Modifier.weight(1f).padding(8.dp)) {
                    AnimatedContent(
                        targetState = selectedTab,
                        transitionSpec = {
                            fadeIn(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) togetherWith 
                            fadeOut(animationSpec = spring(stiffness = Spring.StiffnessMediumLow))
                        },
                        label = "EmojiPickerTabTransition"
                    ) { targetTab ->
                        when (targetTab) {
                            0 -> Column {
                                EmojiGrid(
                                    groups = emojiGroups,
                                    state = gridState,
                                    onEmojiSelected = onEmojiSelected,
                                    modifier = Modifier.weight(1f)
                                )
                                if (emojiGroups.size > 1) {
                                    EmojiServerBar(
                                        groups = emojiGroups,
                                        selectedIndex = selectedGroupIndex,
                                        onSelect = { index ->
                                            selectedGroupIndex = index
                                            coroutineScope.launch {
                                                gridState.animateScrollToItem(groupOffsets[index])
                                            }
                                        }
                                    )
                                }
                            }
                            1 -> Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Text("GIFs coming soon", style = MaterialTheme.typography.bodyMedium)
                            }
                            2 -> Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Text("Stickers coming soon", style = MaterialTheme.typography.bodyMedium)
                            }
                        }
                    }
                }
            }
        }
    }
}

data class EmojiGroup(val id: String?, val guildName: String?, val emojis: List<Emoji>, val iconUrl: String?)

@Composable
fun EmojiGrid(
    groups: List<EmojiGroup>,
    state: LazyGridState,
    onEmojiSelected: (Emoji) -> Unit,
    modifier: Modifier = Modifier
) {
    if (groups.all { it.emojis.isEmpty() }) {
        Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("No custom emojis available", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    } else {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(40.dp),
            state = state,
            modifier = modifier,
            contentPadding = PaddingValues(4.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            groups.forEach { group ->
                if (group.emojis.isNotEmpty()) {
                    item(span = { GridItemSpan(maxLineSpan) }) {
                        Text(
                            text = group.guildName ?: "Unknown Server",
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(top = 8.dp, bottom = 4.dp, start = 4.dp)
                        )
                    }
                    items(group.emojis) { emoji ->
                        val url = emoji.url ?: if (emoji.id != null) {
                            val ext = if (emoji.animated == true) "gif" else "png"
                            "https://cdn.discordapp.com/emojis/${emoji.id}.$ext?size=48"
                        } else null
                        
                        if (url != null) {
                            AsyncImage(
                                model = url,
                                contentDescription = emoji.name,
                                modifier = Modifier
                                    .size(40.dp)
                                    .clip(RoundedCornerShape(4.dp))
                                    .clickable { onEmojiSelected(emoji) }
                                    .padding(4.dp),
                                filterQuality = FilterQuality.Medium
                            )
                        } else if (emoji.name != null) {
                            Box(
                                modifier = Modifier
                                    .size(40.dp)
                                    .clip(RoundedCornerShape(4.dp))
                                    .clickable { onEmojiSelected(emoji) }
                                    .padding(4.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = emoji.name,
                                    fontSize = 24.sp
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun EmojiServerBar(
    groups: List<EmojiGroup>,
    selectedIndex: Int,
    onSelect: (Int) -> Unit
) {
    Column {
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        LazyRow(
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            itemsIndexed(groups) { index, group ->
                val selected = selectedIndex == index
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
                        if (group.iconUrl != null) {
                            AsyncImage(
                                model = group.iconUrl,
                                contentDescription = group.guildName,
                                modifier = Modifier.fillMaxSize()
                            )
                        } else {
                            Icon(
                                imageVector = if (group.id == "standard") Icons.Filled.SentimentSatisfied else Icons.Filled.Group,
                                contentDescription = group.guildName,
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
