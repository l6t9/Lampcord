@file:OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class, kotlin.ExperimentalStdlibApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
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
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import me.lampu.lampcord.shared.model.EmbedImage
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.downloadToDownloads
import me.lampu.lampcord.shared.utils.EmojiIndex
import me.lampu.lampcord.shared.model.toTwemojiUrl
import org.koin.compose.koinInject
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.settings.Settings

@Composable
fun EmojiPicker(
    userStore: UserStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    emojiStore: EmojiStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    mediaApi: MediaApi = koinInject(),
    settingsStore: me.lampu.lampcord.shared.state.SettingsStore = koinInject(),
    modifier: Modifier = Modifier,
    onEmojiSelected: (Emoji) -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    val reduceMotion = Settings.shared.reduceMotion
    val currentUser by userStore.currentUser.collectAsState()
    val nitro = (currentUser?.premium_type ?: 0) > 0 || me.lampu.lampcord.shared.settings.Settings.shared.freeNitroEmojis
    
    var categorizedEmojis: Map<String, List<Emoji>> by remember { mutableStateOf(emptyMap<String, List<Emoji>>()) }
    LaunchedEffect(Unit) {
        categorizedEmojis = EmojiLoader.getCategorizedEmojis()
    }

    val guilds by guildStore.guilds.collectAsState()
    val selectedGuild = navigationStore.selectedGuild
    val emojiGroups: List<EmojiGroup> = remember(selectedGuild, guilds, nitro, categorizedEmojis, emojiStore.frequentEmojis) {
        val groups = mutableListOf<EmojiGroup>()
        val customEmojisById = guilds.flatMap { it.emojis }.associateBy { it.id }
        fun emojiFromKey(key: String): Emoji? {
            // Discord clients have used several equivalent keys over time:
            // name:id, <:name:id>, <a:name:id>, and sometimes just the ID.
            val normalized = key.trim().removePrefix("<a:").removePrefix("<:").removeSuffix(">")
            val parts = normalized.split(":")
            val id = parts.lastOrNull()?.takeIf { it.isNotEmpty() && it.all(Char::isDigit) }
                ?: normalized.takeIf { it.length in 15..22 && it.all(Char::isDigit) }
            return if (id != null) {
                customEmojisById[id] ?: parts.dropLast(1).joinToString(":")
                    .takeIf { it.isNotBlank() }
                    ?.let { name -> Emoji(name = name, id = id, animated = key.startsWith("<a:")) }
            } else {
                val emojiName = normalized.removeSurrounding(":")
                val unicode = EmojiIndex.getCharForName(emojiName)
                when {
                    unicode != null -> Emoji(name = emojiName, url = unicode.toTwemojiUrl())
                    EmojiIndex.getNamesForChar(normalized) != null -> Emoji(name = normalized, url = normalized.toTwemojiUrl())
                    else -> null
                }
            }
        }
        // Favorites from persistent settings
        val favoriteKeys = try {
            val json = Settings.shared.favoriteEmojisJson
            Json.decodeFromString<List<String>>(json)
        } catch (e: Exception) {
            emptyList()
        }
        if (favoriteKeys.isNotEmpty()) {
            val favEmojis = favoriteKeys.mapNotNull(::emojiFromKey)
            if (favEmojis.isNotEmpty()) groups.add(EmojiGroup("favorites", "Favorites", favEmojis, null))
        }

        // Discord stores picker frecency in the account settings proto. Keep
        // the synced order at the top of the picker, then supplement it with
        // emojis used locally in Lampcord.
        val frequentEmojis = emojiStore.frequentEmojis.mapNotNull(::emojiFromKey)
        if (frequentEmojis.isNotEmpty()) {
            groups.add(EmojiGroup("frequent", "Frequently Used", frequentEmojis, null))
        }

        if (nitro) {
            val orderedGuilds = run {
                val settings = settingsStore.userSettings
                if (settings?.guild_folders.isNullOrEmpty()) {
                    guilds
                } else {
                    val out = mutableListOf<me.lampu.lampcord.shared.model.Guild>()
                    val seen = mutableSetOf<String>()
                    settings!!.guild_folders.forEach { folder ->
                        val guildIds = folder.guild_ids.mapNotNull { el -> el.jsonPrimitive.contentOrNull }
                        if (folder.id == null && guildIds.size == 1) {
                            val g = guilds.find { it.id == guildIds.first() }
                            if (g != null) {
                                out.add(g)
                                seen.add(g.id)
                            }
                        } else {
                            guildIds.forEach { id ->
                                val g = guilds.find { it.id == id }
                                if (g != null) {
                                    out.add(g)
                                    seen.add(id)
                                }
                            }
                        }
                    }
                    out.addAll(guilds.filter { it.id !in seen })
                    out
                }
            }

            groups.addAll(
                orderedGuilds
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
    var cloneImageUrl by remember { mutableStateOf<String?>(null) }
    var showCloneModal by remember { mutableStateOf(false) }
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

    var selectedGroupIndex by remember(filteredGroups) { mutableStateOf(0) }

    val favoriteKeys = remember { 
        try {
            Json.decodeFromString<List<String>>(me.lampu.lampcord.shared.settings.Settings.shared.favoriteEmojisJson)
        } catch (e: Exception) { emptyList() }
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
                            if (reduceMotion) {
                                EnterTransition.None togetherWith ExitTransition.None
                            } else {
                                fadeIn(animationSpec = spring(stiffness = Spring.StiffnessMediumLow)) togetherWith
                                fadeOut(animationSpec = spring(stiffness = Spring.StiffnessMediumLow))
                            }
                        },
                        label = "EmojiPickerTabTransition"
                    ) { targetTab ->
                        when (targetTab) {
                            0 -> Column {
                                EmojiGrid(
                                    groups = filteredGroups,
                                                                state = gridState,
                                                                emojiStore = emojiStore,
                                                                navigationStore = navigationStore,
                                                                coroutineScope = coroutineScope,
                                                                onCloneRequested = { url ->
                                                                    cloneImageUrl = url
                                                                    showCloneModal = true
                                                                },
                                                                onEmojiSelected = onEmojiSelected,
                                                                favorites = favoriteKeys,
                                                                onToggleFavorite = { key ->
                                                                    // persist favorites
                                                                    val current = try {
                                                                        Json.decodeFromString<List<String>>(Settings.shared.favoriteEmojisJson).toMutableList()
                                                                    } catch (e: Exception) { mutableListOf() }
                                                                    if (current.contains(key)) current.remove(key) else current.add(0, key)
                                                                    Settings.shared.favoriteEmojisJson = Json.encodeToString(current)
                                                                },
                                                                modifier = Modifier.weight(1f)
                                )
                                if (filteredGroups.size > 1 && searchQuery.isEmpty()) {
                                    EmojiServerBar(
                                        groups = filteredGroups,
                                        selectedIndex = selectedGroupIndex,
                                        onSelect = { index ->
                                            selectedGroupIndex = index
                                            coroutineScope.launch {
                                                if (reduceMotion) gridState.scrollToItem(groupOffsets[index]) else gridState.animateScrollToItem(groupOffsets[index])
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
                                    // Klipy's canonical URL is the shareable
                                    // page URL. Sending its CDN preview URL
                                    // makes Discord post a raw WebP instead of
                                    // a GIF embed.
                                    messageStore.sendMessageDraft(
                                        gif.url.takeIf { it.isNotBlank() }
                                            ?: gif.gifSrc?.takeIf { it.isNotBlank() }
                                            ?: gif.src
                                    )
                                    navigationStore.isEmojiPickerVisible = false
                                }
                            )
                            2 -> StickerPicker(
                                mediaApi = mediaApi,
                                onStickerSelected = { sticker ->
                                    val replacement = me.lampu.lampcord.shared.utils.FreeNitroEmojis.getStickerReplacement(
                                        sticker,
                                        currentUser,
                                        navigationStore.selectedGuild?.id
                                    )
                                    if (replacement != null) {
                                        messageStore.sendMessageDraft(replacement)
                                    } else {
                                        messageStore.sendMessageDraft("", stickerIds = listOf(sticker.id))
                                    }
                                    navigationStore.isEmojiPickerVisible = false
                                }
                            )
                        }
                    }
                }
            }
        }
    }
    
    if (showCloneModal && cloneImageUrl != null) {
        CloneToServerModal(imageUrl = cloneImageUrl!!, defaultName = "emoji", onDismiss = { showCloneModal = false })
    }
}

data class EmojiGroup(val id: String?, val guildName: String?, val emojis: List<Emoji>, val iconUrl: String?)

@Composable
fun EmojiGrid(
    groups: List<EmojiGroup>,
    state: LazyGridState,
    emojiStore: EmojiStore,
    navigationStore: NavigationStore,
    coroutineScope: kotlinx.coroutines.CoroutineScope,
    onCloneRequested: (String) -> Unit,
    onEmojiSelected: (Emoji) -> Unit,
    favorites: List<String> = emptyList(),
    onToggleFavorite: (String) -> Unit = {},
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
                        val displayUrl = if (Settings.shared.reduceMotion &&
                            emoji.animated == true && emoji.id != null
                        ) {
                            // Request a PNG directly instead of allowing the
                            // picker to decode and autoplay the GIF.
                            "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=48"
                        } else {
                            url
                        }
                        val key = if (emoji.id != null) "${emoji.name}:${emoji.id}" else (emoji.name ?: "")
                        val isFav = favorites.contains(key)

                        val menuItems = mutableListOf<ContextMenuItem>()
                        if (url != null) {
                            val u = url
                            menuItems.add(ContextMenuItem("Copy Link", Icons.Filled.Link, onClick = {
                                setClipboardText(u)
                                showToast("Copied to clipboard")
                            }))
                            menuItems.add(ContextMenuItem("View Image", Icons.Filled.OpenInNew, onClick = {
                                val img = EmbedImage(url = u, proxy_url = u)
                                navigationStore.openAttachmentViewer(listOf(img))
                            }))
                        }

                        // Emoji code (custom) or unicode text
                        menuItems.add(ContextMenuItem("Copy Emoji Code", Icons.Filled.ContentCopy, onClick = {
                            val code = if (emoji.id != null) "<${if (emoji.animated == true) "a" else ""}:${emoji.name}:${emoji.id}>" else ":${emoji.name}:"
                            setClipboardText(code)
                            showToast("Copied to clipboard")
                        }))

                        // Save / Clone actions
                        if (url != null) {
                            val filename = (emoji.name ?: "emoji") + if (emoji.animated == true) ".gif" else ".png"
                            menuItems.add(ContextMenuItem("Save Image", Icons.Filled.Download, onClick = {
                                coroutineScope.launch {
                                    val ok = downloadToDownloads(url, filename)
                                    if (ok) showToast("Saved to Downloads") else showToast("Save failed")
                                }
                            }))
                            menuItems.add(ContextMenuItem("Clone to other server", Icons.Filled.Upload, onClick = {
                                onCloneRequested(url)
                            }))
                        }

                        // Favorite toggle moved to context menu
                        menuItems.add(
                            ContextMenuItem(
                                if (isFav) "Remove from Favorites" else "Add to Favorites",
                                if (isFav) Icons.Filled.Favorite else Icons.Rounded.FavoriteBorder,
                                onClick = { onToggleFavorite(key) }
                            )
                        )

                        ContextMenu(items = menuItems) {
                            Box(modifier = Modifier.size(40.dp)) {
                                if (displayUrl != null) {
                                    AsyncImage(
                                        model = displayUrl,
                                        contentDescription = emoji.name,
                                        modifier = Modifier
                                            .fillMaxSize()
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
                                            .fillMaxSize()
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

                                // Favorite action is now available in the context menu
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
