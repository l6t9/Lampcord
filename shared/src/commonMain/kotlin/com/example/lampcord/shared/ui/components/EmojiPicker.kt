@file:OptIn(ExperimentalMaterial3Api::class)

package com.example.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.lampcord.shared.model.Emoji
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.icons.Icons

@Composable
fun EmojiPicker(
    chatState: ChatState,
    onEmojiSelected: (Emoji) -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    val nitro = (chatState.currentUser?.premium_type ?: 0) > 0
    
    var defaultEmojis by remember { mutableStateOf<List<Emoji>>(emptyList()) }
    LaunchedEffect(Unit) {
        defaultEmojis = EmojiLoader.getDefaultEmojis()
    }

    val emojiGroups = remember(chatState.selectedGuild, chatState.guilds.size, nitro, defaultEmojis) {
        val groups = mutableListOf<EmojiGroup>()
        
        if (nitro) {
            groups.addAll(
                chatState.guilds
                    .sortedByDescending { it.id == chatState.selectedGuild?.id }
                    .map { guild ->
                        EmojiGroup(guild.name, guild.emojis)
                    }.filter { it.emojis.isNotEmpty() }
            )
        } else {
            groups.add(EmojiGroup(chatState.selectedGuild?.name, chatState.selectedGuild?.emojis ?: emptyList()))
        }
        
        if (defaultEmojis.isNotEmpty()) {
            groups.add(EmojiGroup("Standard Emojis", defaultEmojis))
        }
        
        groups
    }

    Surface(
        modifier = Modifier.width(400.dp).height(500.dp), // Slightly larger for better Nitro browsing
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shadowElevation = 8.dp
    ) {
        Column {
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
                        0 -> EmojiGrid(emojiGroups, onEmojiSelected)
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

data class EmojiGroup(val guildName: String?, val emojis: List<Emoji>)

@Composable
fun EmojiGrid(groups: List<EmojiGroup>, onEmojiSelected: (Emoji) -> Unit) {
    if (groups.all { it.emojis.isEmpty() }) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("No custom emojis available", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    } else {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(40.dp),
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
