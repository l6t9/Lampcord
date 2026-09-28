package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.UserActivity
import org.koin.compose.koinInject

private val TAB_ACTIVITY = 0
private val TAB_MUTUAL_FRIENDS = 1
private val TAB_MUTUAL_SERVERS = 2

@Composable
fun FullProfileDetails(
    profile: UserProfile,
    theme: ProfileTheme,
    onOpenProfile: (String) -> Unit,
    onOpenGuild: (String) -> Unit,
    modifier: Modifier = Modifier,
    presenceStore: PresenceStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    var selectedTab by remember(profile.user.id) { mutableIntStateOf(TAB_ACTIVITY) }
    val user = profile.user

    val friendsCount = profile.mutual_friends_count ?: profile.mutual_friends?.size ?: 0
    val serversCount = profile.mutual_guilds?.size ?: 0
    val presences by presenceStore.presences.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()
    val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]
    val activities = remember(profile, presence) {
        (profile.activities.ifEmpty { presence?.activities ?: emptyList() }).filter { it.type != 4 }
    }
    val isOwnProfile = user.id == currentUser?.id
    val textColor = theme.customTextColor ?: MaterialTheme.colorScheme.onSurface

    Column(
        modifier = modifier
            .clip(MaterialTheme.shapes.large)
            .background(theme.backgroundBrush)
    ) {
        CompositionLocalProvider(LocalContentColor provides textColor) {
            PrimaryTabRow(
                selectedTabIndex = selectedTab,
                containerColor = Color.Transparent,
                contentColor = textColor,
                divider = {}
            ) {
                Tab(
                    selected = selectedTab == TAB_ACTIVITY,
                    onClick = { selectedTab = TAB_ACTIVITY },
                    text = { Text("Activity", color = textColor) }
                )
                if (!isOwnProfile) {
                    Tab(
                        selected = selectedTab == TAB_MUTUAL_FRIENDS,
                        onClick = { selectedTab = TAB_MUTUAL_FRIENDS },
                        text = { Text("$friendsCount Mutual\nFriends", color = textColor) }
                    )
                    Tab(
                        selected = selectedTab == TAB_MUTUAL_SERVERS,
                        onClick = { selectedTab = TAB_MUTUAL_SERVERS },
                        text = { Text("$serversCount Mutual\nServers", color = textColor) }
                    )
                }
            }

            HorizontalDivider(color = textColor.copy(alpha = 0.15f))

            when (selectedTab) {
                TAB_ACTIVITY -> ActivityTab(activities = activities, modifier = Modifier.weight(1f))
                TAB_MUTUAL_FRIENDS -> MutualFriendsTab(
                    profile = profile,
                    onOpenProfile = onOpenProfile,
                    modifier = Modifier.weight(1f)
                )
                else -> MutualServersTab(
                    profile = profile,
                    onOpenGuild = onOpenGuild,
                    modifier = Modifier.weight(1f)
                )
            }
            }
    }
}

@Composable
private fun ActivityTab(
    activities: List<Activity>,
    modifier: Modifier = Modifier
) {
    if (activities.isEmpty()) {
        EmptyTabState("No recent activity", modifier)
        return
    }
    LazyColumn(
        modifier = modifier.fillMaxWidth(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(activities) { activity ->
            UserActivity(activity = activity, compact = false)
        }
    }
}

@Composable
private fun MutualFriendsTab(
    profile: UserProfile,
    onOpenProfile: (String) -> Unit,
    modifier: Modifier = Modifier,
    userApi: UserApi = koinInject()
) {
    val initial = profile.mutual_friends
    var friends by remember(profile.user.id) { mutableStateOf<List<User>?>(initial) }
    var isLoading by remember(profile.user.id) { mutableStateOf(initial == null) }

    LaunchedEffect(profile.user.id) {
        if (initial != null) return@LaunchedEffect
        isLoading = true
        friends = userApi.getMutualFriends(profile.user.id)
        isLoading = false
    }

    when {
        isLoading -> Box(modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        friends.isNullOrEmpty() -> EmptyTabState("No mutual friends", modifier)
        else -> LazyColumn(modifier = modifier.fillMaxWidth()) {
            items(friends!!) { friend ->
                MutualFriendRow(friend = friend, onClick = { onOpenProfile(friend.id) })
            }
        }
    }
}

@Composable
private fun MutualServersTab(
    profile: UserProfile,
    onOpenGuild: (String) -> Unit,
    modifier: Modifier = Modifier,
    guildStore: GuildStore = koinInject()
) {
    val mutualGuilds = profile.mutual_guilds ?: emptyList()
    val guilds by guildStore.guilds.collectAsState()

    if (mutualGuilds.isEmpty()) {
        EmptyTabState("No mutual servers", modifier)
        return
    }

    LazyColumn(modifier = modifier.fillMaxWidth()) {
        items(mutualGuilds) { mutual ->
            val guild = guilds.find { it.id == mutual.id }
            if (guild != null) {
                MutualServerRow(
                    mutual = mutual,
                    guildStore = guildStore,
                    onClick = { onOpenGuild(guild.id) }
                )
            } else {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .background(LocalContentColor.current.copy(alpha = 0.12f), CircleShape),
                        contentAlignment = Alignment.Center
                    ) {
                        Text("?")
                    }
                    Spacer(Modifier.width(12.dp))
                    Text("Unknown Server")
                    if (mutual.nick != null) {
                        Spacer(Modifier.width(6.dp))
                        Text(
                            text = "(${mutual.nick})",
                            style = MaterialTheme.typography.bodySmall,
                            color = LocalContentColor.current.copy(alpha = 0.7f)
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun EmptyTabState(text: String, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(text, color = LocalContentColor.current.copy(alpha = 0.6f))
    }
}
