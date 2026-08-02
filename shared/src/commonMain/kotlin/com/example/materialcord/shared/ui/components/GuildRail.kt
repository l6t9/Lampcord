package com.example.materialcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.Guild
import com.example.materialcord.shared.model.GuildFolder
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.icons.MaterialcordIcons

@Composable
fun GuildIcon(
    guild: Guild,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    val iconUrl = if (guild.icon != null) {
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png"
    } else null

    Surface(
        modifier = Modifier.size(48.dp),
        onClick = onClick,
        shape = if (isSelected) MaterialTheme.shapes.medium else MaterialTheme.shapes.extraLarge,
        color = if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
    ) {
        if (iconUrl != null) {
            AsyncImage(
                model = iconUrl,
                contentDescription = guild.name,
                modifier = Modifier.fillMaxSize()
            )
        } else {
            Box(contentAlignment = Alignment.Center) {
                Text(
                    text = guild.name?.take(1) ?: "?",
                    color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun GuildFolderItem(folder: GuildFolder, chatState: ChatState) {
    var expanded by remember { mutableStateOf(false) }
    val folderColor = folder.color?.let { Color(it.toLong() or 0xFF000000L) } ?: Color(0xFF5865F2)

    Surface(
        color = if (expanded) MaterialTheme.colorScheme.surface else Color.Transparent,
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier.width(52.dp)
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.padding(vertical = 4.dp)
        ) {
            Surface(
                modifier = Modifier.size(48.dp),
                onClick = { expanded = !expanded },
                shape = MaterialTheme.shapes.extraLarge,
                color = if (expanded) Color.Transparent else folderColor.copy(alpha = 0.2f)
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = if (expanded) MaterialcordIcons.Filled.FolderOpen else MaterialcordIcons.Filled.Folder,
                        contentDescription = folder.name ?: "Folder",
                        tint = if (expanded) MaterialTheme.colorScheme.primary else folderColor,
                        modifier = Modifier.size(24.dp)
                    )
                }
            }

            if (expanded) {
                Column(
                    modifier = Modifier.padding(top = 8.dp, bottom = 4.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    folder.guild_ids.forEach { guildId ->
                        val guild = chatState.guilds.find { it.id == guildId }
                        if (guild != null) {
                            GuildIcon(
                                guild = guild,
                                isSelected = chatState.selectedGuild?.id == guild.id,
                                onClick = { chatState.selectGuild(guild) }
                            )
                        }
                    }
                }
            }
        }
    }
}
