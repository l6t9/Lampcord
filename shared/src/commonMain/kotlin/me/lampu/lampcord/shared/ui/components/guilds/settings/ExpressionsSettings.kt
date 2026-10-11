package me.lampu.lampcord.shared.ui.components.guilds.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.material3.Surface
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
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Sticker
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.customEmojiCdnUrl
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import org.koin.compose.koinInject
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

@OptIn(ExperimentalEncodingApi::class)
@Composable
fun ServerEmoji(guild: Guild, guildApi: GuildApi = koinInject()) {
    var emojis by remember { mutableStateOf(guild.emojis) }
    var isLoading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    
    var showUploadPicker by remember { mutableStateOf(false) }
    var emojiToRename by remember { mutableStateOf<Emoji?>(null) }
    var newEmojiName by remember { mutableStateOf("") }

    LaunchedEffect(guild.id) {
        isLoading = true
        emojis = guildApi.getGuildEmojis(guild.id)
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
                val newEmoji = guildApi.createEmoji(guild.id, file.first.substringBeforeLast("."), dataUrl)
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
                        val updated = guildApi.updateEmoji(guild.id, emojiToRename!!.id!!, newEmojiName)
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
        Material3SettingsGroup(title = "Emojis") {
            Button(
                onClick = { showUploadPicker = true },
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                shape = RoundedCornerShape(8.dp)
            ) {
                Icon(Icons.Rounded.Upload, null, Modifier.size(18.dp))
                Spacer(Modifier.width(8.dp))
                Text("Upload Emoji")
            }

            if (isLoading) {
                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            } else if (emojis.isEmpty()) {
                Text("No custom emojis", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
            } else {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    emojis.forEach { emoji ->
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
                            onClick = {
                                newEmojiName = emoji.name ?: ""
                                emojiToRename = emoji
                            }
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                AsyncImage(
                                    model = emoji.id?.let { customEmojiCdnUrl(it, emoji.animated == true, size = 128) },
                                    contentDescription = emoji.name,
                                    allowAnimation = emoji.animated == true,
                                    modifier = Modifier.size(36.dp)
                                )
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(emoji.name ?: "unnamed", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                                    Text("Added by ${emoji.user?.global_name ?: emoji.user?.username ?: "Unknown"}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                IconButton(onClick = {
                                    scope.launch {
                                        if (guildApi.deleteEmoji(guild.id, emoji.id!!)) {
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
        Spacer(modifier = Modifier.height(100.dp))
    }
}

@Composable
fun ServerStickers(guild: Guild, guildApi: GuildApi = koinInject()) {
    var stickers by remember { mutableStateOf<List<Sticker>>(guild.stickers) }
    var isLoading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    var showUploadPicker by remember { mutableStateOf(false) }
    var stickerToEdit by remember { mutableStateOf<Sticker?>(null) }
    var newStickerName by remember { mutableStateOf("") }
    var newStickerDescription by remember { mutableStateOf("") }
    var newStickerTags by remember { mutableStateOf("") }

    LaunchedEffect(guild.id) {
        isLoading = true
        stickers = guildApi.getGuildStickers(guild.id)
        isLoading = false
    }

    FilePicker(
        show = showUploadPicker,
        onDismiss = { showUploadPicker = false },
        onFileSelected = { files ->
            val file = files.firstOrNull() ?: return@FilePicker
            scope.launch {
                val filename = file.first
                val bytes = file.second
                val name = filename.substringBeforeLast(".")
                val created = guildApi.createSticker(guild.id, name, null, null, filename to bytes)
                if (created != null) {
                    stickers = stickers + created
                }
            }
        }
    )

    SettingsLayout {
        Material3SettingsGroup(title = "Stickers") {
            Button(
                onClick = { showUploadPicker = true },
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                shape = RoundedCornerShape(8.dp)
            ) {
                Icon(Icons.Rounded.Upload, null, Modifier.size(18.dp))
                Spacer(Modifier.width(8.dp))
                Text("Upload Sticker")
            }

            if (isLoading) {
                Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                    ContainedLoadingIndicator()
                }
            } else if (stickers.isEmpty()) {
                Text("No custom stickers", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
            } else {
                Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    stickers.forEach { sticker ->
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
                            onClick = {
                                newStickerName = sticker.name
                                newStickerDescription = sticker.description ?: ""
                                newStickerTags = sticker.tags ?: ""
                                stickerToEdit = sticker
                            }
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                AsyncImage(
                                    model = "https://cdn.discordapp.com/stickers/${sticker.id}.png?size=96",
                                    contentDescription = sticker.name,
                                    modifier = Modifier.size(36.dp)
                                )
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(sticker.name, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                                    Text(sticker.description ?: "", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                IconButton(onClick = {
                                    scope.launch {
                                        val guildId = sticker.guild_id ?: return@launch
                                        if (guildApi.deleteSticker(guildId, sticker.id)) {
                                            stickers = stickers.filter { it.id != sticker.id }
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
        Spacer(modifier = Modifier.height(100.dp))
    }

    if (stickerToEdit != null) {
        AlertDialog(
            onDismissRequest = { stickerToEdit = null },
            title = { Text("Edit Sticker") },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    OutlinedTextField(value = newStickerName, onValueChange = { newStickerName = it }, label = { Text("Name") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    OutlinedTextField(value = newStickerDescription, onValueChange = { newStickerDescription = it }, label = { Text("Description") }, modifier = Modifier.fillMaxWidth())
                    OutlinedTextField(value = newStickerTags, onValueChange = { newStickerTags = it }, label = { Text("Tags") }, modifier = Modifier.fillMaxWidth())
                }
            },
            confirmButton = {
                Button(onClick = {
                    val s = stickerToEdit!!
                    scope.launch {
                        val updated = guildApi.updateSticker(s.guild_id ?: return@launch, s.id, name = newStickerName, description = newStickerDescription, tags = newStickerTags)
                        if (updated != null) {
                            stickers = stickers.map { if (it.id == updated.id) updated else it }
                        }
                        stickerToEdit = null
                    }
                }) { Text("Save") }
            },
            dismissButton = {
                TextButton(onClick = { stickerToEdit = null }) { Text("Cancel") }
            }
        )
    }
}
