package com.example.materialcord.shared.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.Guild
import com.example.materialcord.shared.model.Message
import com.example.materialcord.shared.state.ChatState
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    chatState: ChatState = koinInject()
) {
    var tokenInput by remember { mutableStateOf("") }
    var showTokenDialog by remember { mutableStateOf(!chatState.isConnected) }

    if (showTokenDialog && !chatState.isConnected) {
        AlertDialog(
            onDismissRequest = { },
            title = { Text("Enter Discord Token") },
            text = {
                TextField(
                    value = tokenInput,
                    onValueChange = { tokenInput = it },
                    placeholder = { Text("mfa.xxxxx...") }
                )
            },
            confirmButton = {
                Button(onClick = {
                    chatState.connect(tokenInput)
                    showTokenDialog = false
                }) {
                    Text("Connect")
                }
            }
        )
    }

    Row(modifier = Modifier.fillMaxSize()) {
        // Sidebar (Guilds)
        Column(
            modifier = Modifier
                .width(80.dp)
                .fillMaxHeight()
                .background(MaterialTheme.colorScheme.surfaceContainerLow)
                .padding(vertical = 12.dp),
            horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            // Discord Logo / Home / DMs
            Surface(
                modifier = Modifier.size(48.dp),
                onClick = {
                    chatState.selectedGuild = null
                    chatState.selectedChannel = null
                },
                shape = if (chatState.selectedGuild == null) MaterialTheme.shapes.medium else MaterialTheme.shapes.extraLarge,
                color = if (chatState.selectedGuild == null) Color(0xFF5865F2) else MaterialTheme.colorScheme.surfaceVariant
            ) {
                Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                    Text(
                        text = "D",
                        color = if (chatState.selectedGuild == null) Color.White else MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.titleLarge
                    )
                }
            }

            HorizontalDivider(modifier = Modifier.width(32.dp))

            // Guild icons
            if (chatState.guilds.isEmpty()) {
                repeat(5) {
                    GuildIconPlaceholder(it + 1)
                }
            } else {
                LazyColumn(
                    horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(chatState.guilds) { guild ->
                        GuildIcon(
                            guild = guild,
                            isSelected = chatState.selectedGuild?.id == guild.id,
                            onClick = { chatState.selectGuild(guild) }
                        )
                    }
                }
            }
            
            Spacer(modifier = Modifier.weight(1f))
            
            // Current User Avatar
            chatState.currentUser?.let { user ->
                val userAvatarUrl = user.avatar?.let { 
                    "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
                }
                Surface(
                    modifier = Modifier.size(40.dp),
                    shape = MaterialTheme.shapes.extraLarge,
                    color = MaterialTheme.colorScheme.secondaryContainer
                ) {
                    if (userAvatarUrl != null) {
                        AsyncImage(model = userAvatarUrl, contentDescription = "Me")
                    } else {
                        Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                            Text(user.username.take(1).uppercase())
                        }
                    }
                }
            }
        }

        // Sidebar (Channels)
        if (chatState.selectedGuild != null) {
            Column(
                modifier = Modifier
                    .width(240.dp)
                    .fillMaxHeight()
                    .background(MaterialTheme.colorScheme.surfaceContainer)
                    .padding(8.dp)
            ) {
                Text(
                    text = chatState.selectedGuild?.name ?: "Guild",
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(8.dp)
                )
                HorizontalDivider()
                Spacer(modifier = Modifier.height(8.dp))
                LazyColumn {
                    items(chatState.channels) { channel ->
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            onClick = { chatState.selectChannel(channel) },
                            color = if (chatState.selectedChannel?.id == channel.id) 
                                MaterialTheme.colorScheme.surfaceVariant 
                            else Color.Transparent,
                            shape = MaterialTheme.shapes.small
                        ) {
                            Text(
                                text = "# ${channel.name}",
                                modifier = Modifier.padding(8.dp),
                                style = MaterialTheme.typography.bodyMedium
                            )
                        }
                    }
                }
            }
        }

        // Chat Area
        Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
            TopAppBar(
                title = { 
                    Column {
                        Text(chatState.selectedChannel?.name ?: "Materialcord")
                        Text(
                            if (chatState.isConnected) "Connected" else "Disconnected",
                            style = MaterialTheme.typography.labelSmall
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface
                )
            )
            
            ChatArea(
                modifier = Modifier.weight(1f),
                chatState = chatState
            )
            
            // Message Input
            var messageText by remember { mutableStateOf("") }

            if (chatState.selectedChannel != null) {
                Row(modifier = Modifier.fillMaxWidth().padding(8.dp)) {
                    TextField(
                        value = messageText,
                        onValueChange = { messageText = it },
                        modifier = Modifier.weight(1f),
                        placeholder = { Text("Message #${chatState.selectedChannel?.name}") },
                        trailingIcon = {
                            IconButton(onClick = {
                                if (messageText.isNotBlank()) {
                                    chatState.sendMessage(messageText)
                                    messageText = ""
                                }
                            }) {
                                Text("Send")
                            }
                        }
                    )
                }
            }
        }
    }
}

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
            Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                Text(
                    text = guild.name?.take(1) ?: "?",
                    color = if (isSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun GuildIconPlaceholder(index: Int) {
    Surface(
        modifier = Modifier.size(48.dp),
        shape = MaterialTheme.shapes.extraLarge,
        color = MaterialTheme.colorScheme.surfaceVariant
    ) {
        Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
            Text("$index")
        }
    }
}

@Composable
fun ChatArea(
    modifier: Modifier = Modifier,
    chatState: ChatState
) {
    LazyColumn(
        modifier = modifier.fillMaxSize().padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        reverseLayout = true
    ) {
        items(chatState.messages) { message ->
            MessageItem(message)
        }
    }
}

@Composable
fun MessageItem(message: Message) {
    Row(modifier = Modifier.fillMaxWidth()) {
        val avatarUrl = message.author.avatar?.let { 
            "https://cdn.discordapp.com/avatars/${message.author.id}/$it.png"
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = "Avatar",
                modifier = Modifier.size(40.dp).clip(MaterialTheme.shapes.extraLarge)
            )
        } else {
            Surface(
                modifier = Modifier.size(40.dp),
                shape = MaterialTheme.shapes.extraLarge,
                color = MaterialTheme.colorScheme.primaryContainer
            ) {
                Box(contentAlignment = androidx.compose.ui.Alignment.Center) {
                    Text(
                        message.author.username.take(1).uppercase(),
                        style = MaterialTheme.typography.titleMedium
                    )
                }
            }
        }
        
        Spacer(modifier = Modifier.width(12.dp))
        
        Column {
            Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Text(
                    text = message.author.global_name ?: message.author.username,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.primary
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = message.timestamp.take(10), // Simplistic timestamp
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Text(
                text = message.content,
                style = MaterialTheme.typography.bodyLarge
            )
        }
    }
}
