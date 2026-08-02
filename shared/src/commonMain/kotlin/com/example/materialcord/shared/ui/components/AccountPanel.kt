package com.example.materialcord.shared.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.icons.MaterialcordIcons

@Composable
fun AccountPanel(chatState: ChatState) {
    var showMenu by remember { mutableStateOf(false) }
    
    chatState.currentUser?.let { user ->
        val userAvatarUrl = user.avatar?.let { 
            "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
        }
        
        Surface(
            modifier = Modifier.fillMaxWidth().height(68.dp),
            onClick = { user.id.let { chatState.showProfile(it) } },
            color = MaterialTheme.colorScheme.surfaceContainer,
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box {
                    Surface(
                        modifier = Modifier.size(38.dp),
                        shape = CircleShape,
                        color = MaterialTheme.colorScheme.secondaryContainer
                    ) {
                        if (userAvatarUrl != null) {
                            AsyncImage(model = userAvatarUrl, contentDescription = "Me", modifier = Modifier.fillMaxSize())
                        } else {
                            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.labelLarge)
                            }
                        }
                    }
                    
                    // Status dot
                    Surface(
                        modifier = Modifier.size(12.dp).align(Alignment.BottomEnd),
                        shape = CircleShape,
                        color = Color(0xFF43B581),
                        border = BorderStroke(2.dp, MaterialTheme.colorScheme.surfaceContainer)
                    ) {}
                }
                
                Spacer(modifier = Modifier.width(12.dp))
                
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = user.global_name ?: user.username,
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = "Online",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                
                IconButton(
                    onClick = { /* TODO: Open Settings */ },
                    colors = IconButtonDefaults.filledTonalIconButtonColors()
                ) {
                    Icon(
                        imageVector = MaterialcordIcons.Filled.Settings,
                        contentDescription = "Settings",
                        modifier = Modifier.size(20.dp)
                    )
                }
            }

            DropdownMenu(
                expanded = showMenu,
                onDismissRequest = { showMenu = false }
            ) {
                DropdownMenuItem(
                    text = {
                        Column {
                            Text(user.global_name ?: user.username, style = MaterialTheme.typography.titleSmall)
                            Text("@${user.username}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    },
                    onClick = { },
                    enabled = false
                )
                HorizontalDivider()
                DropdownMenuItem(
                    leadingIcon = { Icon(MaterialcordIcons.Filled.Settings, null, modifier = Modifier.size(18.dp)) },
                    text = { Text("User Settings") },
                    onClick = { showMenu = false }
                )
                DropdownMenuItem(
                    leadingIcon = { Text("🚪") },
                    text = { Text("Log Out") },
                    onClick = { 
                        showMenu = false
                    }
                )
            }
        }
    }
}
