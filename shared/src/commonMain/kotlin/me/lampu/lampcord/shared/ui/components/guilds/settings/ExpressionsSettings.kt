package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.components.settings.SettingsSection
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import org.koin.compose.koinInject
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun ServerEmoji(guild: Guild, discordClient: DiscordClient = koinInject()) {
    var emojis by remember { mutableStateOf(guild.emojis) }
    var isLoading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    
    var showUploadPicker by remember { mutableStateOf(false) }
    var emojiToRename by remember { mutableStateOf<Emoji?>(null) }
    var newEmojiName by remember { mutableStateOf("") }

    LaunchedEffect(guild.id) {
        isLoading = true
        emojis = discordClient.getGuildEmojis(guild.id)
        isLoading = false
    }

    FilePicker(
        show = showUploadPicker,
        onDismiss = { showUploadPicker = false },
        onFileSelected = { files ->
            val file = files.firstOrNull() ?: return@FilePicker
            scope.launch {
                val extension = file.first.substringAfterLast(".", "png")
                val mimeType = if (extension == "gif") "image/gif" else "image/png"
                val base64Data = Base64.encode(file.second)
                val dataUrl = "data:$mimeType;base64,$base64Data"
                val newEmoji = discordClient.createEmoji(guild.id, file.first.substringBeforeLast("."), dataUrl)
                if (newEmoji != null) {
                    emojis = emojis + newEmoji
                }
            }
        }
    )

    if (emojiToRename != null) {
        AlertDialog(
            onDismissRequest = { emojiToRename = null },
            title = { Text("Rename Emoji") },
            text = {
                OutlinedTextField(
                    value = newEmojiName,
                    onValueChange = { newEmojiName = it },
                    label = { Text("Emoji Name") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            confirmButton = {
                Button(onClick = {
                    scope.launch {
                        val updated = discordClient.updateEmoji(guild.id, emojiToRename!!.id!!, newEmojiName)
                        if (updated != null) {
                            emojis = emojis.map { if (it.id == updated.id) updated else it }
                        }
                        emojiToRename = null
                    }
                }) {
                    Text("Save")
                }
            },
            dismissButton = {
                TextButton(onClick = { emojiToRename = null }) {
                    Text("Cancel")
                }
            }
        )
    }

    SettingsLayout {
        SettingsSection(
            title = "Emoji",
            icon = Icons.Filled.Mood,
            actions = {
                Button(onClick = { showUploadPicker = true }) {
                    Icon(Icons.Rounded.Upload, null, Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("Upload Emoji")
                }
            }
        ) {
            if (isLoading) {
                ContainedLoadingIndicator(modifier = Modifier.align(Alignment.CenterHorizontally))
            } else if (emojis.isEmpty()) {
                Text("No custom emojis", color = MaterialTheme.colorScheme.onSurfaceVariant)
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    emojis.forEach { emoji ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 56.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
                                .clickable { 
                                    newEmojiName = emoji.name ?: ""
                                    emojiToRename = emoji 
                                }
                                .padding(horizontal = 16.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            AsyncImage(
                                model = "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=96",
                                contentDescription = emoji.name,
                                modifier = Modifier.size(32.dp)
                            )
                            Column(modifier = Modifier.weight(1f)) {
                                Text(emoji.name ?: "unnamed", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                                Text("Added by ${emoji.user?.global_name ?: emoji.user?.username ?: "Unknown"}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            IconButton(onClick = {
                                scope.launch {
                                    if (discordClient.deleteEmoji(guild.id, emoji.id!!)) {
                                        emojis = emojis.filter { it.id != emoji.id }
                                    }
                                }
                            }) {
                                Icon(Icons.Default.Delete, "Delete", tint = MaterialTheme.colorScheme.error)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ServerStickers() {
    SettingsLayout {
        SettingsSection(title = "Stickers", icon = Icons.Filled.StickyNote2) {
            Text("Sticker management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
