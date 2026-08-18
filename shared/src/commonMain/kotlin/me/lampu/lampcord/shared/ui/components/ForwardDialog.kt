package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.FinderResult
import me.lampu.lampcord.shared.state.FinderStore
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ForwardDialog(
    message: Message,
    onDismiss: () -> Unit,
    finderStore: FinderStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    var comment by remember { mutableStateOf("") }
    
    LaunchedEffect(Unit) {
        finderStore.searchQuery = ""
    }

    val resultsState by finderStore.results.collectAsState()
    val results by remember(finderStore.searchQuery, resultsState) {
        derivedStateOf {
            resultsState.filter { it !is FinderResult.Guild }
        }
    }

    Dialog(
        onDismissRequest = onDismiss,
        properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
    ) {
        Surface(
            modifier = Modifier
                .widthIn(max = 600.dp)
                .fillMaxWidth(0.95f)
                .heightIn(max = 850.dp)
                .fillMaxHeight(0.9f)
                .clip(MaterialTheme.shapes.large),
            shape = MaterialTheme.shapes.large,
            color = MaterialTheme.colorScheme.surfaceContainerLowest,
        ) {
            Column {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Text(
                        text = "Forward Message",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.align(Alignment.CenterStart)
                    )
                    IconButton(
                        onClick = onDismiss,
                        modifier = Modifier.align(Alignment.CenterEnd)
                    ) {
                        Icon(Icons.Default.Close, "Close", tint = MaterialTheme.colorScheme.onSurface, modifier = Modifier.size(24.dp))
                    }
                }

                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .padding(horizontal = 16.dp)
                ) {
                    // Message Preview
                    Text(
                        "Message Preview",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                    
                    Surface(
                        color = MaterialTheme.colorScheme.surfaceContainerLow,
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.Top) {
                            AsyncImage(
                                model = CdnUrls.getUserAvatarUrl(message.author?.id ?: "", message.author?.avatar),
                                contentDescription = null,
                                modifier = Modifier.size(32.dp).clip(CircleShape)
                            )
                            Spacer(Modifier.width(12.dp))
                            Column {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text(
                                        message.author?.global_name ?: message.author?.username ?: "Unknown User",
                                        style = MaterialTheme.typography.bodyMedium,
                                        fontWeight = FontWeight.Bold
                                    )
                                }
                                if (message.content.isNotEmpty()) {
                                    Text(
                                        message.content,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurface,
                                        maxLines = 3,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                                if (message.attachments.isNotEmpty()) {
                                    Row(
                                        modifier = Modifier.padding(top = 4.dp),
                                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                                    ) {
                                        message.attachments.take(3).forEach { attachment ->
                                            if (attachment.content_type?.startsWith("image/") == true) {
                                                AsyncImage(
                                                    model = attachment.proxy_url,
                                                    contentDescription = null,
                                                    modifier = Modifier.size(48.dp).clip(RoundedCornerShape(4.dp)),
                                                    contentScale = ContentScale.Crop
                                                )
                                            } else {
                                                Surface(
                                                    color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                                    shape = RoundedCornerShape(4.dp),
                                                    modifier = Modifier.size(48.dp)
                                                ) {
                                                    Icon(
                                                        Icons.Filled.Description,
                                                        null,
                                                        modifier = Modifier.padding(12.dp).size(24.dp),
                                                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                                                    )
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Spacer(Modifier.height(16.dp))

                    // Optional Message
                    Text(
                        "Optional Message",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                    TextField(
                        value = comment,
                        onValueChange = { comment = it },
                        modifier = Modifier.fillMaxWidth(),
                        placeholder = { Text("Add a comment...") },
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            focusedIndicatorColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent
                        ),
                        shape = MaterialTheme.shapes.medium,
                    )

                    Spacer(Modifier.height(16.dp))

                    // Forward To
                    Text(
                        "FORWARD TO",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                    TextField(
                        value = finderStore.searchQuery,
                        onValueChange = { finderStore.searchQuery = it },
                        modifier = Modifier.fillMaxWidth(),
                        placeholder = { Text("Search for a channel or DM") },
                        leadingIcon = { Icon(Icons.Filled.Search, null) },
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                            focusedIndicatorColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent
                        ),
                        shape = MaterialTheme.shapes.medium,
                        singleLine = true
                    )

                    Spacer(Modifier.height(8.dp))

                    Surface(
                        shape = MaterialTheme.shapes.medium,
                        color = MaterialTheme.colorScheme.surfaceContainerLow,
                        modifier = Modifier.fillMaxWidth().weight(1f)
                    ) {
                        LazyColumn(
                            modifier = Modifier.fillMaxWidth(),
                            contentPadding = PaddingValues(8.dp),
                            verticalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            items(results) { result ->
                                FinderResultItem(result, onClick = {
                                    val targetChannel = when (result) {
                                        is FinderResult.Channel -> result.channel
                                        is FinderResult.DirectMessage -> result.channel
                                        else -> null
                                    }
                                    targetChannel?.let {
                                        messageStore.forwardMessage(it, message)
                                        if (comment.isNotEmpty()) {
                                            val currentUser = userStore.currentUser.value
                                            if (currentUser != null) {
                                                messageStore.sendMessage(
                                                    channelId = it.id,
                                                    content = comment,
                                                    currentUser = currentUser,
                                                    guildId = it.guild_id
                                                )
                                            }
                                        }
                                        onDismiss()
                                    }
                                })
                            }
                            
                            if (results.isEmpty()) {
                                item {
                                    Box(Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                                        Text("No results found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    }
                                }
                            }
                        }
                    }
                    Spacer(Modifier.height(16.dp))
                }
            }
        }
    }
}

@Composable
private fun FinderResultItem(result: FinderResult, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        onClick = onClick,
        shape = RoundedCornerShape(16.dp),
        color = Color.Transparent
    ) {
        Row(
            modifier = Modifier.padding(8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val icon = when (result) {
                is FinderResult.Channel -> if (result.channel.type == 5) Icons.Filled.Campaign else Icons.Filled.Tag
                is FinderResult.DirectMessage -> Icons.Filled.Person
                else -> Icons.Filled.Tag
            }
            
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(Modifier.width(12.dp))
            
            Column {
                val title = when (result) {
                    is FinderResult.Channel -> result.channel.name ?: "unnamed-channel"
                    is FinderResult.DirectMessage -> result.channel.recipients?.firstOrNull()?.let { it.global_name ?: it.username } ?: "Direct Message"
                    is FinderResult.Guild -> result.guild.name ?: "Unnamed Server"
                }
                Text(title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                
                val subtitle = when (result) {
                    is FinderResult.Channel -> result.guild?.name ?: "Server"
                    is FinderResult.DirectMessage -> "Direct Message"
                    is FinderResult.Guild -> "Server"
                }
                Text(subtitle, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}
