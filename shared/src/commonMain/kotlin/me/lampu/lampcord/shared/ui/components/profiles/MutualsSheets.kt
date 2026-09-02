package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.MutualGuild
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AdaptiveModalBottomSheet
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.LoadingIndicators
import me.lampu.lampcord.shared.ui.components.StatusIndicator
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun MutualFriendsBottomSheet(
    userId: String,
    username: String,
    onDismiss: () -> Unit,
    userApi: UserApi = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    var mutualFriends by remember { mutableStateOf<List<User>?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    val currentUser by userStore.currentUser.collectAsState()

    LaunchedEffect(userId) {
        isLoading = true
        mutualFriends = userApi.getMutualFriends(userId)
        isLoading = false
    }

    AdaptiveModalBottomSheet(onDismissRequest = onDismiss) {
        Column(modifier = Modifier.fillMaxWidth().padding(bottom = 16.dp)) {
            Text(
                text = "Mutual Friends",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(16.dp)
            )

            if (isLoading) {
                Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            } else {
                val friends = mutualFriends ?: emptyList()
                if (friends.isEmpty()) {
                    Box(Modifier.fillMaxWidth().height(100.dp), contentAlignment = Alignment.Center) {
                        Text("No mutual friends", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                } else {
                    LazyColumn(modifier = Modifier.fillMaxWidth().heightIn(max = 400.dp)) {
                        items(friends) { friend ->
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clickable { /* TODO: Open profile? */ }
                                    .padding(horizontal = 16.dp, vertical = 8.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Box(modifier = Modifier.size(40.dp)) {
                                    AsyncImage(
                                        model = "https://cdn.discordapp.com/avatars/${friend.id}/${friend.avatar}.png?size=128",
                                        contentDescription = null,
                                        modifier = Modifier.fillMaxSize().clip(CircleShape)
                                    )
                                    Box(
                                        modifier = Modifier
                                            .align(Alignment.BottomEnd)
                                            .size(14.dp)
                                            .background(MaterialTheme.colorScheme.surface, CircleShape)
                                            .padding(2.dp)
                                    ) {
                                        StatusIndicator(
                                            status = presenceStore.getUserStatus(friend.id, currentUser?.id, null),
                                            size = 10.dp,
                                            borderWidth = 0.dp
                                        )
                                    }
                                }
                                Spacer(Modifier.width(12.dp))
                                Column {
                                    Text(
                                        text = friend.global_name ?: friend.username ?: "Unknown",
                                        style = MaterialTheme.typography.bodyLarge,
                                        fontWeight = FontWeight.Bold
                                    )
                                    Text(
                                        text = friend.username ?: "",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun MutualServersBottomSheet(
    userId: String,
    username: String,
    mutualGuilds: List<MutualGuild>,
    onDismiss: () -> Unit,
    guildStore: GuildStore = koinInject()
) {
    val allGuilds by guildStore.guilds.collectAsState()

    AdaptiveModalBottomSheet(onDismissRequest = onDismiss) {
        Column(modifier = Modifier.fillMaxWidth().padding(bottom = 16.dp)) {
            Text(
                text = "Mutual Servers",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(16.dp)
            )

            if (mutualGuilds.isEmpty()) {
                Box(Modifier.fillMaxWidth().height(100.dp), contentAlignment = Alignment.Center) {
                    Text("No mutual servers", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            } else {
                LazyColumn(modifier = Modifier.fillMaxWidth().heightIn(max = 400.dp)) {
                    items(mutualGuilds) { mutual ->
                        val guild = allGuilds.find { it.id == mutual.id }
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { /* TODO: Navigate to guild? */ }
                                .padding(horizontal = 16.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (guild?.icon != null) {
                                AsyncImage(
                                    model = "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=128",
                                    contentDescription = null,
                                    modifier = Modifier.size(40.dp).clip(CircleShape)
                                )
                            } else {
                                Box(
                                    modifier = Modifier
                                        .size(40.dp)
                                        .background(MaterialTheme.colorScheme.surfaceVariant, CircleShape),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Text(
                                        text = guild?.name?.take(1) ?: "?",
                                        style = MaterialTheme.typography.titleMedium
                                    )
                                }
                            }
                            Spacer(Modifier.width(12.dp))
                            Column {
                                Text(
                                    text = guild?.name ?: "Unknown Server",
                                    style = MaterialTheme.typography.bodyLarge,
                                    fontWeight = FontWeight.Bold
                                )
                                if (mutual.nick != null) {
                                    Text(
                                        text = "Nickname: ${mutual.nick}",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
