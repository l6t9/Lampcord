package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.api.GuildApi
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.utils.fetchUrlBytes
import me.lampu.lampcord.shared.utils.base64Encode
import me.lampu.lampcord.shared.utils.showToast

@Composable
fun CloneToServerModal(
    imageUrl: String,
    defaultName: String,
    onDismiss: () -> Unit
) {
    val guildStore: GuildStore = koinInject()
    val guilds by guildStore.guilds.collectAsState()
    val guildApi: GuildApi = koinInject()
    val coroutineScope = rememberCoroutineScope()
    var name by remember { mutableStateOf(defaultName) }

    Surface(modifier = Modifier.fillMaxWidth().padding(16.dp)) {
        Column(modifier = Modifier.fillMaxWidth()) {
            Text("Clone to Server", style = MaterialTheme.typography.titleMedium)
            OutlinedTextField(value = name, onValueChange = { name = it }, label = { Text("Emoji name") }, modifier = Modifier.fillMaxWidth().padding(top = 8.dp, bottom = 8.dp))
            Text("Choose target server:", style = MaterialTheme.typography.labelMedium)
            LazyColumn(modifier = Modifier.fillMaxWidth().heightIn(max = 300.dp)) {
                items(guilds) { guild ->
                    ListItem(
                        headlineContent = { Text(guild.name ?: "Unknown") },
                        supportingContent = { Text(guild.id) },
                        modifier = Modifier.clickable {
                            coroutineScope.launch {
                                val bytes = fetchUrlBytes(imageUrl)
                                if (bytes == null) {
                                    showToast("Failed to download image")
                                    return@launch
                                }
                                val dataUrl = "data:image/png;base64,${base64Encode(bytes)}"
                                val created = guildApi.createEmoji(guild.id, name, dataUrl)
                                if (created != null) {
                                    showToast("Cloned to ${guild.name}")
                                } else {
                                    showToast("Failed to clone to ${guild.name}")
                                }
                                onDismiss()
                            }
                        }
                    )
                    Divider()
                }
            }
            Spacer(Modifier.height(8.dp))
            Button(onClick = onDismiss) { Text("Cancel") }
        }
    }
}
