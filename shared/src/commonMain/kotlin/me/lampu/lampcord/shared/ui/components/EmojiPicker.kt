@file:OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)

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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.coerceAtMost
import androidx.compose.ui.zIndex
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.MediaApi
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Gif
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import androidx.compose.foundation.layout.ExperimentalLayoutApi

@Composable
fun EmojiPicker(
    userStore: UserStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    emojiStore: EmojiStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    mediaApi: MediaApi = koinInject(),
    modifier: Modifier = Modifier,
    onEmojiSelected: (Emoji) -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    val currentUser by userStore.currentUser.collectAsState()
    val nitro = (currentUser?.premium_type ?: 0) > 0 || me.lampu.lampcord.shared.settings.Settings.shared.freeNitroEmojis
    
    var categorizedEmojis by remember { mutableStateOf<Map<String, List<Emoji>>>(emptyMap()) }
    LaunchedEffect(Unit) {
        categorizedEmojis = EmojiLoader.getCategorizedEmojis()
    }

    val guilds by guildStore.guilds.collectAsState()
    val selectedGuild = navigationStore.selectedGuild
    val emojiGroups = remember(selectedGuild, guilds.size, nitro, categorizedEmojis, emojiStore.frequentEmojis) {
        val groups = mutableListOf<EmojiGroup>()
        
        if (emojiStore.frequentEmojis.isNotEmpty()) {
            val frequent = emojiStore.frequentEmojis.map { key ->
                if (key.contains(":")) {
                    val parts = key.split(":")
                    Emoji(name = parts[0], id = parts[1])
                } else {
                    Emoji(name = key)
                }
            }
            groups.add(EmojiGroup("frequent", "Frequently Used", frequent, null))
        }

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
        
        categorizedEmojis.forEach { (category, emojis) ->
            val displayName = when (category) {
                "people" -> "Smileys & People"
                "nature" -> "Animals & Nature"
                "food" -> "Food & Drink"
                "activity" -> "Activities"
                "travel" -> "Travel & Places"
                "objects" -> "Objects"
                "symbols" -> "Symbols"
                "flags" -> "Flags"
                else -> category.replaceFirstChar { it.uppercase() }
            }
            groups.add(EmojiGroup("standard_$category", displayName, emojis, null))
        }
        
        groups
    }

    val gridState = rememberLazyGridState()
    val coroutineScope = rememberCoroutineScope()
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
    val searchFocusRequester = remember { FocusRequester() }

    val imeHeight = WindowInsets.ime.asPaddingValues().calculateBottomPadding()
    val keyboardHeight = remember(imeHeight) { 
        if (imeHeight > 0.dp) imeHeight else 360.dp
    }

    var searchQuery by remember { mutableStateOf("") }
    
    LaunchedEffect(navigationStore.isEmojiPickerVisible) {
        if (navigationStore.isEmojiPickerVisible && !isMobile) {
            searchFocusRequester.requestFocus()
        }
    }

    val filteredGroups = remember(searchQuery, emojiGroups) {
        if (searchQuery.isBlank()) emojiGroups
        else {
            emojiGroups.map { group ->
                group.copy(emojis = group.emojis.filter { it.name?.contains(searchQuery, ignoreCase = true) == true })
            }.filter { it.emojis.isNotEmpty() }
        }
    }

    val groupOffsets = remember(filteredGroups) {
        val offsets = mutableListOf<Int>()
        var acc = 0
        filteredGroups.forEach { group ->
            offsets.add(acc)
            acc += 1 + group.emojis.size
        }
        offsets
    }

    var selectedGroupIndex by remember(emojiGroups) { mutableStateOf(0) }

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
        modifier = if (isMobile) {
            modifier
                .fillMaxWidth()
                .height(keyboardHeight.coerceAtMost(600.dp))
        } else {
            modifier.width(400.dp).height(500.dp)
        },
        shape = if (isMobile) RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp) else RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shadowElevation = if (isMobile) 0.dp else 16.dp
    ) {
        Row(modifier = Modifier.fillMaxSize()) {
            Column(modifier = Modifier.weight(1f)) {
                SecondaryTabRow(
                    selectedTabIndex = selectedTab,
                    containerColor = Color.Transparent,
                    contentColor = MaterialTheme.colorScheme.primary,
                    divider = {},
                ) {
                    Tab(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        selectedContentColor = MaterialTheme.colorScheme.primary,
                        unselectedContentColor = MaterialTheme.colorScheme.onSurfaceVariant
                    ) {
                        Box(Modifier.padding(12.dp)) {
                            Text("Emoji", style = MaterialTheme.typography.labelLarge, fontWeight = if (selectedTab == 0) FontWeight.Bold else FontWeight.Normal)
                        }
                    }
                    Tab(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        selectedContentColor = MaterialTheme.colorScheme.primary,
                        unselectedContentColor = MaterialTheme.colorScheme.onSurfaceVariant
                    ) {
                        Box(Modifier.padding(12.dp)) {
                            Text("GIFs", style = MaterialTheme.typography.labelLarge, fontWeight = if (selectedTab == 1) FontWeight.Bold else FontWeight.Normal)
                        }
                    }
                    Tab(
                        selected = selectedTab == 2,
                        onClick = { selectedTab = 2 },
                        selectedContentColor = MaterialTheme.colorScheme.primary,
                        unselectedContentColor = MaterialTheme.colorScheme.onSurfaceVariant
                    ) {
                        Box(Modifier.padding(12.dp)) {
                            Text("Stickers", style = MaterialTheme.typography.labelLarge, fontWeight = if (selectedTab == 2) FontWeight.Bold else FontWeight.Normal)
                        }
                    }
                }

                if (selectedTab == 0 || selectedTab == 1) {
                    TextField(
                        value = searchQuery,
                        onValueChange = { searchQuery = it },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 12.dp, vertical = 4.dp)
                            .heightIn(min = 44.dp)
                            .focusRequester(searchFocusRequester),
                        placeholder = { 
                            Text(
                                if (selectedTab == 0) "Find the perfect emoji" else "Search GIFs (Klipy)", 
                                style = MaterialTheme.typography.bodyMedium, 
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                            ) 
                        },
                        leadingIcon = { Icon(Icons.Filled.Search, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)) },
                        trailingIcon = if (searchQuery.isNotEmpty()) {
                            {
                                IconButton(onClick = { searchQuery = "" }) {
                                    Icon(Icons.Filled.Close, null, modifier = Modifier.size(18.dp))
                                }
                            }
                        } else null,
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            focusedIndicatorColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent,
                            disabledIndicatorColor = Color.Transparent
                        ),
                        shape = RoundedCornerShape(16.dp),
                        singleLine = true,
                        textStyle = MaterialTheme.typography.bodyMedium
                    )
                }

                Box(modifier = Modifier.weight(1f).padding(16.dp)) {
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
                                    groups = filteredGroups,
                                    state = gridState,
                                    emojiStore = emojiStore,
                                    onEmojiSelected = onEmojiSelected,
                                    modifier = Modifier.weight(1f)
                                )
                                if (filteredGroups.size > 1 && searchQuery.isEmpty()) {
                                    EmojiServerBar(
                                        groups = filteredGroups,
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
                            1 -> GifPicker(
                                query = searchQuery,
                                onQueryChange = { searchQuery = it },
                                mediaApi = mediaApi,
                                onGifSelected = { gif ->
                                    messageStore.sendMessageDraft(gif.url)
                                    navigationStore.isEmojiPickerVisible = false
                                }
                            )
                            2 -> StickerPicker(
                                mediaApi = mediaApi,
                                onStickerSelected = { sticker ->
                                    messageStore.sendMessageDraft("", stickerIds = listOf(sticker.id))
                                    navigationStore.isEmojiPickerVisible = false
                                }
                            )
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
    emojiStore: EmojiStore,
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
                            modifier = Modifier.padding(top = 16.dp, bottom = 4.dp, start = 4.dp)
                        )
                    }
                    items(group.emojis) { emoji ->
                        val url = emoji.getDisplayUrl()
                        
                        if (url != null) {
                            AsyncImage(
                                model = url,
                                contentDescription = emoji.name,
                                modifier = Modifier
                                    .size(40.dp)
                                    .clip(RoundedCornerShape(4.dp))
.clickable { 
                                                        val emojiStr = if (emoji.id != null) "${emoji.name}:${emoji.id}" else ":${emoji.name}:"
                                                        emojiStore.onEmojiUsed(emojiStr)
                                                        onEmojiSelected(emoji) 
                                                    }
                                    .padding(4.dp),
                                filterQuality = FilterQuality.Medium
                            )
                        } else if (emoji.name != null) {
                            Box(
                                modifier = Modifier
                                    .size(40.dp)
                                    .clip(RoundedCornerShape(4.dp))
.clickable { 
                                                        val emojiName = emoji.name
                                                        val emojiStr = if (emoji.id != null) "${emojiName}:${emoji.id}" else ":${emojiName}:"
                                                        emojiStore.onEmojiUsed(emojiStr)
                                                        onEmojiSelected(emoji) 
                                                    }
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
            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp),
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
                                imageVector = when (group.id) {
                                    "frequent" -> Icons.Filled.History
                                    "standard_people" -> Icons.Filled.SentimentSatisfied
                                    "standard_nature" -> Icons.Filled.EmojiNature
                                    "standard_food" -> Icons.Filled.EmojiFoodBeverage
                                    "standard_activity" -> Icons.Filled.SportsEsports
                                    "standard_travel" -> Icons.Filled.EmojiTransportation
                                    "standard_objects" -> Icons.Filled.EmojiObjects
                                    "standard_symbols" -> Icons.Filled.EmojiSymbols
                                    "standard_flags" -> Icons.Filled.Flag
                                    else -> Icons.Filled.Group
                                },
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
