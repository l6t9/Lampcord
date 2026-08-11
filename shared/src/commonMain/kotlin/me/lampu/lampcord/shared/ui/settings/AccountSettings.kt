package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun AccountSettings(userStore: UserStore = koinInject()) {
    val user by userStore.currentUser.collectAsState()
    val userVal = user ?: return
    
    SettingsLayout {
        SettingsSection(
            title = "Account Information",
            icon = Icons.Filled.AccountCircle
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    val avatarUrl = userVal.avatar?.let { "https://cdn.discordapp.com/avatars/${userVal.id}/$it.png?size=128" }
                    if (avatarUrl != null) {
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = "Avatar",
                            modifier = Modifier.size(80.dp).clip(CircleShape)
                        )
                    } else {
                        Surface(
                            modifier = Modifier.size(80.dp),
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.primaryContainer
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text(userVal.username?.take(1)?.uppercase() ?: "?", style = MaterialTheme.typography.headlineMedium)
                            }
                        }
                    }

                    Spacer(Modifier.width(20.dp))

                    Column {
                        Text(userVal.global_name ?: userVal.username ?: "Unknown User", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                        Text(userVal.username ?: "", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }

                HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    AccountInfoItem("Username", userVal.username ?: "Not set")
                    AccountInfoItem("Email", userVal.email ?: "Not set")
                }

                Spacer(Modifier.height(8.dp))

                Button(onClick = { /* TODO */ }) {
                    Text("Edit User Profile")
                }
            }
        }

        SettingsSection(
            title = "Password and Authentication",
            icon = Icons.Filled.Lock
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Material3SettingsItem(
                    title = { Text("Change Password") },
                    onClick = { /* TODO */ }
                )
                Material3SettingsItem(
                    title = { Text("Two-Factor Authentication") },
                    description = { 
                        Text(
                            text = if (userVal.mfa_enabled == true) "Two-Factor Authentication: Enabled" else "Two-Factor Authentication: Disabled",
                            color = if (userVal.mfa_enabled == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    },
                    onClick = { /* TODO */ }
                )
            }
        }
    }
}

@Composable
private fun AccountInfoItem(label: String, value: String) {
    Column {
        Text(label.uppercase(), style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.ExtraBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurface)
    }
}
