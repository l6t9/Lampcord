package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.components.guilds.GuildChannelList
import me.lampu.lampcord.shared.ui.components.guilds.GuildRail
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun Sidebar(
    navigationStore: NavigationStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    modifier: Modifier = Modifier
) {
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"

    Column(
        modifier = modifier.fillMaxHeight(),
        verticalArrangement = Arrangement.Top
    ) {
        Row(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Surface(
                modifier = Modifier.fillMaxHeight().width(72.dp),
                color = Color.Transparent,
                tonalElevation = 0.dp
            ) {
                GuildRail()
            }

            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight()
            ) {
                // Channels / DMs List
                Surface(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth(),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.surface,
                    tonalElevation = 0.dp
                ) {
                    if (navigationStore.selectedGuild != null) {
                        GuildChannelList()
                    } else {
                        DMList()
                    }
                }
            }
        }

        if (voiceStore.isVoiceConnected) {
            Spacer(Modifier.height(8.dp))
            VoiceConnectionPanel()
        }

        if (isMobile) {
            Spacer(Modifier.height(8.dp))
            NavigationBar(
                modifier = Modifier.fillMaxWidth().height(64.dp).clip(RoundedCornerShape(16.dp)),
                containerColor = MaterialTheme.colorScheme.surface,
                tonalElevation = 0.dp
            ) {
                NavigationBarItem(
                    selected = !navigationStore.isFriendsSelected && !navigationStore.isSettingsVisible && !navigationStore.isSearchVisible,
                    onClick = {
                        navigationStore.isFriendsSelected = false
                        navigationStore.isSettingsVisible = false
                        navigationStore.isSearchVisible = false
                    },
                    icon = { Icon(Icons.Brand.Discord, "Home") }
                )
                NavigationBarItem(
                    selected = navigationStore.isFriendsSelected,
                    onClick = { navigationStore.selectFriends() },
                    icon = { Icon(if (navigationStore.isFriendsSelected) Icons.Filled.Person else Icons.Rounded.Person, "Friends") }
                )
                NavigationBarItem(
                    selected = navigationStore.isSearchVisible,
                    onClick = { navigationStore.isSearchVisible = true },
                    icon = { Icon(Icons.Filled.Search, "Search") }
                )
                NavigationBarItem(
                    selected = navigationStore.isSettingsVisible,
                    onClick = { navigationStore.isSettingsVisible = true },
                    icon = { Icon(Icons.Filled.Settings, "Settings") }
                )
            }
        } else {
            Spacer(Modifier.height(8.dp))
            // Account Panel (CurrentUser) spans both GuildRail and ChannelsList
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(60.dp),
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surface,
                tonalElevation = 0.dp
            ) {
                AccountPanel()
            }
        }
    }
}
