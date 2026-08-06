package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun AccountSettings(chatState: ChatState) {
    val user = chatState.currentUser ?: return
    
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Profile") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        leadingContent = {
                            val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=128" }
                            Surface(
                                modifier = Modifier.size(48.dp),
                                shape = CircleShape,
                                color = MaterialTheme.colorScheme.primaryContainer
                            ) {
                                if (avatarUrl != null) {
                                    AsyncImage(model = avatarUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                                } else {
                                    Box(contentAlignment = Alignment.Center) {
                                        Text(user.username.take(1).uppercase())
                                    }
                                }
                            }
                        },
                        title = { Text(user.global_name ?: user.username) },
                        description = { Text(user.username) },
                        trailingContent = {
                            FilledTonalButton(onClick = { /* TODO */ }, modifier = Modifier.height(32.dp)) {
                                Text("Edit")
                            }
                        }
                    )
                ),
                horizontalPadding = 0.dp
            )
        }

        Material3SettingsGroup(title = "Account Information") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Username") },
                        description = { Text(user.username) },
                        onClick = { /* TODO */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Email") },
                        description = { Text("********@gmail.com") },
                        onClick = { /* TODO */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Phone Number") },
                        description = { Text("********1234") },
                        onClick = { /* TODO */ }
                    )
                ),
                horizontalPadding = 0.dp
            )
        }

        Material3SettingsGroup(title = "Password and Authentication") {
            Material3SettingsGroup(
                items = listOf(
                    navigationSettingsItem(
                        title = "Change Password",
                        onClick = { /* TODO */ }
                    ),
                    navigationSettingsItem(
                        title = "Two-Factor Authentication",
                        description = "Protect your account with an extra layer of security",
                        onClick = { /* TODO */ }
                    )
                ),
                horizontalPadding = 0.dp
            )
        }
    }
}
