package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun ProfileSettings(chatState: ChatState) {
    val user = chatState.currentUser ?: return
    
    var displayName by remember { mutableStateOf(user.global_name ?: "") }
    var pronouns by remember { mutableStateOf(user.pronouns ?: "") }
    var bio by remember { mutableStateOf(user.bio ?: "") }

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Preview") {
            Surface(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                color = MaterialTheme.colorScheme.surfaceContainer,
                shape = RoundedCornerShape(24.dp),
                border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)
            ) {
                Column {
                    Box(modifier = Modifier.fillMaxWidth().height(100.dp).background(MaterialTheme.colorScheme.primaryContainer)) {
                        if (user.banner != null) {
                            AsyncImage(
                                model = "https://cdn.discordapp.com/banners/${user.id}/${user.banner}.png?size=600",
                                contentDescription = null,
                                modifier = Modifier.fillMaxSize(),
                                contentScale = ContentScale.Crop
                            )
                        }
                    }
                    
                    Box(modifier = Modifier.padding(horizontal = 16.dp)) {
                        Surface(
                            modifier = Modifier
                                .offset(y = (-40).dp)
                                .size(80.dp),
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.surface,
                            border = androidx.compose.foundation.BorderStroke(4.dp, MaterialTheme.colorScheme.surfaceContainer)
                        ) {
                            val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160" }
                            AvatarWithDecoration(
                                avatarUrl = avatarUrl,
                                decorationData = user.avatar_decoration_data ?: user.collectibles?.avatar_decoration,
                                size = 72.dp,
                                status = "online"
                            )
                        }
                        
                        Column(modifier = Modifier.padding(top = 44.dp, bottom = 16.dp)) {
                            Text(displayName.ifBlank { user.username ?: "" }, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                            Text(user.username ?: "", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            
                            if (pronouns.isNotBlank()) {
                                Spacer(Modifier.height(8.dp))
                                Text(pronouns, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            
                            if (bio.isNotBlank()) {
                                Spacer(Modifier.height(12.dp))
                                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                                Spacer(Modifier.height(12.dp))
                                Text("ABOUT ME", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold)
                                Text(bio, style = MaterialTheme.typography.bodyMedium)
                            }
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(16.dp))

        Material3SettingsGroup(title = "Edit Profile") {
            Column(modifier = Modifier.padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                OutlinedTextField(
                    value = displayName,
                    onValueChange = { displayName = it },
                    label = { Text("Display Name") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )

                OutlinedTextField(
                    value = pronouns,
                    onValueChange = { pronouns = it },
                    label = { Text("Pronouns") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )

                OutlinedTextField(
                    value = bio,
                    onValueChange = { bio = it },
                    label = { Text("About Me") },
                    modifier = Modifier.fillMaxWidth(),
                    minLines = 3,
                    maxLines = 5
                )

                Button(
                    onClick = { /* TODO: Save profile via API */ },
                    modifier = Modifier.align(Alignment.End)
                ) {
                    Text("Save Changes")
                }
            }
        }
    }
}
