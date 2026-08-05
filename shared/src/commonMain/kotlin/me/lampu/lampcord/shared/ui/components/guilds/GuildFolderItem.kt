package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.GuildFolder
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.AsyncImage

@Composable
fun FolderPreviewGrid(folder: GuildFolder, chatState: ChatState) {
    val guilds = folder.guild_ids.mapNotNull { id -> chatState.guilds.find { it.id == id } }.take(4)
    
    Column(
        modifier = Modifier.padding(4.dp).fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(2.dp)
    ) {
        Row(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            Box(Modifier.weight(1f).fillMaxHeight()) {
                if (guilds.size > 0) PreviewIcon(guilds[0])
            }
            Box(Modifier.weight(1f).fillMaxHeight()) {
                if (guilds.size > 1) PreviewIcon(guilds[1])
            }
        }
        Row(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            Box(Modifier.weight(1f).fillMaxHeight()) {
                if (guilds.size > 2) PreviewIcon(guilds[2])
            }
            Box(Modifier.weight(1f).fillMaxHeight()) {
                if (guilds.size > 3) PreviewIcon(guilds[3])
            }
        }
    }
}

@Composable
fun PreviewIcon(guild: Guild) {
    val iconUrl = if (guild.icon != null) {
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=48"
    } else null

    Surface(
        modifier = Modifier.fillMaxSize(),
        shape = RoundedCornerShape(4.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shadowElevation = 0.dp,
        tonalElevation = 0.dp
    ) {
        if (iconUrl != null) {
            AsyncImage(
                model = iconUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                filterQuality = FilterQuality.Low
            )
        } else {
            Box(contentAlignment = Alignment.Center) {
                Text(
                    text = guild.name?.take(1) ?: "?",
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 8.sp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun GuildFolderItem(folder: GuildFolder, chatState: ChatState) {
    var expanded by remember { mutableStateOf(false) }
    val folderColor = folder.color?.let { Color(it.toLong() or 0xFF000000L) } ?: MaterialTheme.colorScheme.primary
    
    val isAnyChildSelected = folder.guild_ids.any { it == chatState.selectedGuild?.id }

    val surfaceColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
    val expansionProgress by animateFloatAsState(targetValue = if (expanded) 1f else 0f)

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .fillMaxWidth()
            .drawBehind {
                if (expansionProgress > 0f) {
                    val wellWidth = 48.dp.toPx()
                    val x = (size.width - wellWidth) / 2
                    drawRoundRect(
                        color = surfaceColor,
                        topLeft = Offset(x, 0f),
                        size = Size(wellWidth, size.height * expansionProgress),
                        cornerRadius = CornerRadius(12.dp.toPx())
                    )
                }
            }
    ) {
        // Folder Icon Row
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp),
            contentAlignment = Alignment.Center
        ) {
            // Indicator for COLLAPSED folder containing the selection
            if (!expanded && isAnyChildSelected) {
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterStart)
                        .width(4.dp)
                        .height(8.dp)
                        .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                        .background(MaterialTheme.colorScheme.onSurface)
                )
            }

            val folderCornerRadius by animateDpAsState(targetValue = if (expanded) 12.dp else 16.dp)
            val folderBgColor by animateColorAsState(targetValue = if (expanded) Color.Transparent else folderColor.copy(alpha = 0.2f))

            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(folderCornerRadius))
                    .background(if (expanded) folderColor.copy(alpha = 0.1f) else folderBgColor)
                    .clickable { expanded = !expanded },
                contentAlignment = Alignment.Center
            ) {
                if (expanded) {
                    Icon(
                        imageVector = Icons.Filled.FolderOpen,
                        contentDescription = folder.name ?: "Folder",
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(28.dp)
                    )
                } else {
                    FolderPreviewGrid(folder, chatState)
                }
            }
        }

        AnimatedVisibility(
            visible = expanded,
            enter = expandVertically(animationSpec = spring(stiffness = 300f)) + fadeIn(),
            exit = shrinkVertically(animationSpec = spring(stiffness = 300f)) + fadeOut()
        ) {
            // Expanded area
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 12.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                folder.guild_ids.forEach { guildId ->
                    val guild = chatState.guilds.find { it.id == guildId }
                    if (guild != null) {
                        GuildIcon(
                            guild = guild,
                            isSelected = chatState.selectedGuild?.id == guild.id,
                            chatState = chatState,
                            onClick = { chatState.selectGuild(guild) }
                        )
                    }
                }
            }
        }
    }
}
