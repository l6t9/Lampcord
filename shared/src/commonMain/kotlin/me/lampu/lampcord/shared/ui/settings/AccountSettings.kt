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
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun AccountSettings(chatState: ChatState) {
    val user = chatState.currentUser ?: return
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopAccountSettings(chatState, user)
    } else {
        MobileAccountSettings(chatState, user)
    }
}

@Composable
private fun DesktopAccountSettings(chatState: ChatState, user: me.lampu.lampcord.shared.model.User) {
    DesktopSettingsLayout {
        DesktopSettingsSection(
            title = "Profile",
            icon = Icons.Filled.AccountCircle
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=128" }
                Surface(
                    modifier = Modifier.size(80.dp),
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.primaryContainer
                ) {
                    if (avatarUrl != null) {
                        AsyncImage(model = avatarUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                    } else {
                        Box(contentAlignment = Alignment.Center) {
                            Text(user.username?.take(1)?.uppercase() ?: "?", style = MaterialTheme.typography.headlineMedium)
                        }
                    }
                }
                
                Column(modifier = Modifier.weight(1f)) {
                    Text(user.global_name ?: user.username ?: "Unknown User", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                    Text(user.username ?: "", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }

                Button(onClick = { /* TODO */ }) {
                    Text("Edit Profile")
                }
            }
        }

        DesktopSettingsSection(
            title = "Account Information",
            icon = Icons.Filled.Info
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                AccountInfoItem("Username", user.username ?: "")
                AccountInfoItem("Email", user.email ?: "Not set")
                AccountInfoItem("Phone Number", "********1234")
            }
        }

        DesktopSettingsSection(
            title = "Password and Authentication",
            icon = Icons.Filled.Security
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Button(onClick = { /* TODO */ }) {
                    Text("Change Password")
                }
                
                Text(
                    text = if (user.mfa_enabled == true) "Two-Factor Authentication: Enabled" else "Two-Factor Authentication: Disabled",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (user.mfa_enabled != true) {
                    Text("Protect your account with an extra layer of security.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Button(onClick = { /* TODO */ }) {
                        Text("Enable 2FA")
                    }
                }
            }
        }
    }
}

@Composable
private fun AccountInfoItem(label: String, value: String) {
    Column {
        Text(label, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
        Text(value, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
private fun MobileAccountSettings(chatState: ChatState, user: me.lampu.lampcord.shared.model.User) {
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
                                        Text(user.username?.take(1)?.uppercase() ?: "?")
                                    }
                                }
                            }
                        },
                        title = { Text(user.global_name ?: user.username ?: "Unknown User") },
                        description = { Text(user.username ?: "") },
                        trailingContent = {
                            FilledTonalButton(onClick = { /* TODO */ }, modifier = Modifier.height(32.dp)) {
                                Text("Edit")
                            }
                        }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Account Information") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Username") },
                        description = { Text(user.username ?: "") },
                        onClick = { /* TODO */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Email") },
                        description = { Text(user.email ?: "Not set") },
                        onClick = { /* TODO */ }
                    ),
                    Material3SettingsItem(
                        title = { Text("Phone Number") },
                        description = { Text("********1234") }, // Discord doesn't return full phone usually
                        onClick = { /* TODO */ }
                    )
                )
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
                        description = if (user.mfa_enabled == true) "Enabled" else "Protect your account with an extra layer of security",
                        onClick = { /* TODO */ }
                    )
                )
            )
        }
    }
}
