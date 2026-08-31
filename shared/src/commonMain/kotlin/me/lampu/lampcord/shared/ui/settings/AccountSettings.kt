package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun AccountSettings(
    onBack: () -> Unit,
    userStore: UserStore = koinInject()
) {
    val user by userStore.currentUser.collectAsState()
    val userVal = user ?: return
    
    SettingsSubScreen(
        title = "Account",
        onNavigateBack = onBack
    ) {
        AccountSettingsContent(userVal = userVal)
    }
}

@Composable
fun AccountSettingsContent(userVal: me.lampu.lampcord.shared.model.User) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Account Information",
            items = listOf(
                Material3SettingsItem(
                    leadingContent = {
                        AsyncImage(
                            model = CdnUrls.getUserAvatarUrl(userVal.id, userVal.avatar, 128),
                            contentDescription = "Avatar",
                            modifier = Modifier.size(60.dp).clip(CircleShape)
                        )
                    },
                    title = { Text(userVal.global_name ?: userVal.username ?: "Unknown User", fontWeight = FontWeight.Bold) },
                    description = { Text(userVal.username ?: "") },
                    trailingContent = {
                        Button(onClick = { /* TODO */ }) {
                            Text("Edit")
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Username") },
                    description = { Text(userVal.username ?: "Not set") }
                ),
                Material3SettingsItem(
                    title = { Text("Email") },
                    description = { Text(userVal.email ?: "Not set") }
                )
            )
        )

        Material3SettingsGroup(
            title = "Password and Authentication",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Change Password") },
                    onClick = { /* TODO */ }
                ),
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
            )
        )
    }
}
