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
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.MutualGuild
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AdaptiveModalBottomSheet
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.StatusIndicator
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Logging
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MutualFriendsBottomSheet(
    userId: String,
    username: String,
    onDismiss: () -> Unit,
    initialFriends: List<User>? = null,
    userApi: UserApi = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    userStore: UserStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
) {
    var mutualFriends by remember { mutableStateOf<List<User>?>(initialFriends) }
    var isLoading by remember { mutableStateOf(initialFriends == null) }
    val currentUser by userStore.currentUser.collectAsState()

    LaunchedEffect(userId) {
        if (initialFriends != null) return@LaunchedEffect
        isLoading = true
        Logging.d("MutualFriends", "Fetching mutual friends for $userId")
        mutualFriends = userApi.getMutualFriends(userId)
        Logging.d("MutualFriends", "Found ${mutualFriends?.size ?: 0} mutual friends")
        isLoading = false
    }

    AdaptiveModalBottomSheet(onDismissRequest = onDismiss, peekHeight = 300.dp) {
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
                                    .clickable { 
                                        onDismiss()
                                        profileStore.showProfile(friend.id)
                                    }
                                    .padding(horizontal = 16.dp, vertical = 8.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Box(modifier = Modifier.size(40.dp)) {
                                    AsyncImage(
                                        model = CdnUrls.getUserAvatarUrl(friend.id, friend.avatar, 128),
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
                                            status = presenceStore.getUserStatus(friend.id, currentUser?.id, settingsStore.userSettings?.status),
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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MutualServersBottomSheet(
    userId: String,
    username: String,
    mutualGuilds: List<MutualGuild>,
    onDismiss: () -> Unit,
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val allGuilds by guildStore.guilds.collectAsState()

    AdaptiveModalBottomSheet(onDismissRequest = onDismiss, peekHeight = 300.dp) {
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
                                .clickable { 
                                    onDismiss()
                                    if (guild != null) {
                                        navigationStore.selectedGuild = guild
                                    }
                                }
                                .padding(horizontal = 16.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (guild?.icon != null) {
                                AsyncImage(
                                    model = CdnUrls.getGuildIconUrl(guild.id, guild.icon, 128),
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
