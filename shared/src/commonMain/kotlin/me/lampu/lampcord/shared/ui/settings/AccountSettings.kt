package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage

@Composable
fun AccountSettings(chatState: ChatState) {
    val user = chatState.currentUser ?: return
    
    Column(modifier = Modifier.fillMaxWidth().widthIn(max = 600.dp)) {
        Text("My Account", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(24.dp))
        
        Surface(
            modifier = Modifier.fillMaxWidth(),
            color = MaterialTheme.colorScheme.surfaceContainer,
            shape = RoundedCornerShape(16.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
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
                                Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.headlineSmall)
                            }
                        }
                    }
                    
                    Spacer(modifier = Modifier.width(24.dp))
                    
                    Column {
                        Text(user.global_name ?: user.username, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                        Text(user.username, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    
                    Spacer(modifier = Modifier.weight(1f))
                    
                    Button(onClick = { /* TODO */ }) {
                        Text("Edit User Profile")
                    }
                }
                
                Spacer(modifier = Modifier.height(24.dp))
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                Spacer(modifier = Modifier.height(16.dp))
                
                SettingsRow("Username", user.username) { /* Edit */ }
                SettingsRow("Email", "********@gmail.com") { /* Edit */ }
                SettingsRow("Phone Number", "********1234") { /* Edit */ }
            }
        }
    }
}

@Composable
fun SettingsRow(label: String, value: String, onEdit: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column {
            Text(label, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Text(value, style = MaterialTheme.typography.bodyLarge)
        }
        Spacer(modifier = Modifier.weight(1f))
        FilledTonalButton(onClick = onEdit, modifier = Modifier.height(32.dp)) {
            Text("Edit", style = MaterialTheme.typography.labelMedium)
        }
    }
}
