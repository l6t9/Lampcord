package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerSettings(chatState: ChatState, onDismiss: () -> Unit) {
    val guild = chatState.selectedGuild ?: return

    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.surface
    ) {
        Column {
            CenterAlignedTopAppBar(
                title = { Text(guild.name ?: "Server Settings") },
                navigationIcon = {
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.Filled.Close, null)
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color.Transparent
                )
            )
            
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                item {
                    Text("Overview", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                }
                
                item {
                    OutlinedCard(
                        modifier = Modifier.fillMaxWidth(),
                        colors = CardDefaults.outlinedCardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow)
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Text("Server Name", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(guild.name ?: "Unknown", style = MaterialTheme.typography.bodyLarge)
                            
                            Spacer(Modifier.height(16.dp))
                            
                            Text("Server ID", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(guild.id, style = MaterialTheme.typography.bodyLarge)
                        }
                    }
                }

                item {
                    Text("Roles", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                }

                items(guild.roles.sortedByDescending { it.position }) { role ->
                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        color = Color.Transparent,
                        onClick = { /* TODO: Edit Role */ }
                    ) {
                        Row(
                            modifier = Modifier.padding(12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(12.dp)
                                    .background(if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurfaceVariant, CircleShape)
                            )
                            Spacer(Modifier.width(12.dp))
                            Text(text = role.name, style = MaterialTheme.typography.bodyLarge)
                            Spacer(Modifier.weight(1f))
                            if (role.managed) {
                                Icon(Icons.Filled.Security, null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                }
                
                item {
                    Text("More settings coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}
