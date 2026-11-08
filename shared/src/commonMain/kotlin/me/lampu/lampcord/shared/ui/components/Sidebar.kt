package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.guilds.GuildChannelList
import me.lampu.lampcord.shared.ui.components.guilds.GuildRail
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun Sidebar(
    navigationStore: NavigationStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    userStore: UserStore = koinInject(),
    modifier: Modifier = Modifier,
    showAccountPanel: Boolean = getPlatformName() != "android" && getPlatformName() != "ios"
) {

    Column(
        modifier = modifier.fillMaxHeight().statusBarsPadding(),
        verticalArrangement = Arrangement.Top
    ) {
        Row(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.Start
        ) {
            Surface(
                modifier = Modifier.fillMaxHeight().width(72.dp),
                color = Color.Transparent,
                tonalElevation = 0.dp
            ) {
                GuildRail()
            }

            Surface(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp),
                color = MaterialTheme.colorScheme.surface,
                tonalElevation = 0.dp
            ) {
                when {
                    navigationStore.isFavoritesSelected && me.lampu.lampcord.shared.settings.Settings.shared.favoritesServerEnabled -> {
                        FavoritesChannelList()
                    }
                    navigationStore.selectedGuild != null -> {
                        GuildChannelList()
                    }
                    else -> {
                        DMList()
                    }
                }
            }
        }

        if (showAccountPanel) {
            Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp)) {
                androidx.compose.animation.AnimatedVisibility(
                    visible = voiceStore.activeChannel != null,
                    enter = androidx.compose.animation.expandVertically(expandFrom = androidx.compose.ui.Alignment.Top) + androidx.compose.animation.fadeIn(),
                    exit = androidx.compose.animation.shrinkVertically(shrinkTowards = androidx.compose.ui.Alignment.Top) + androidx.compose.animation.fadeOut()
                ) {
                    SidebarVoicePanel(voiceStore, userStore)
                }
                Box(
                    modifier = Modifier
                        .padding(start = 0.dp, top = 4.dp, end = 0.dp, bottom = 0.dp)
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(16.dp))
                ) {
                    AccountPanel()
                }
            }
        }
    }
}

@Composable
fun SidebarVoicePanel(
    voiceStore: VoiceStore,
    userStore: UserStore
) {
    val activeChannel = voiceStore.activeChannel ?: return
    val guildStore: GuildStore = koinInject()
    val settingsStore: SettingsStore = koinInject()
    val guilds by guildStore.guilds.collectAsState()
    val guild = remember(activeChannel.guild_id, guilds) {
        guilds.find { it.id == activeChannel.guild_id }
    }
    val connected = voiceStore.isVoiceConnected
    val name = if (guild != null) "${activeChannel.name ?: "Voice"} / ${guild.name}" else voiceChannelName(activeChannel, userStore)

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 4.dp),
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh
    ) {
        Column(modifier = Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        if (connected) "Voice Connected" else "Connecting",
                        style = MaterialTheme.typography.labelLarge,
                        color = if (connected) Color(0xFF6DD58C) else MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.SemiBold
                    )
                    Text(
                        "$name · ${voiceStatusText(voiceStore)}",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                IconButton(
                    onClick = { voiceStore.disconnectFromVoice() },
                    modifier = Modifier.size(36.dp)
                ) {
                    Icon(Icons.Filled.CallEnd, "Disconnect", tint = MaterialTheme.colorScheme.error, modifier = Modifier.size(20.dp))
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                // Sending camera and screen video is not implemented yet; only receiving is.
                VoicePanelButton(Icons.Filled.Videocam, "Turn on camera", enabled = false) {}
                VoicePanelButton(Icons.Filled.ScreenShare, "Share your screen", enabled = false) {}
                VoicePanelButton(
                    Icons.Filled.GraphicEq,
                    if (settingsStore.noiseCancellation) "Turn off noise suppression" else "Turn on noise suppression",
                    checked = settingsStore.noiseCancellation
                ) { settingsStore.noiseCancellation = !settingsStore.noiseCancellation }
            }
        }
    }
}

@Composable
private fun RowScope.VoicePanelButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    contentDescription: String,
    enabled: Boolean = true,
    checked: Boolean = false,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier.weight(1f).height(36.dp),
        shape = CircleShape,
        color = if (checked) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainerHighest,
        contentColor = if (checked) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurface
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(
                icon,
                contentDescription,
                modifier = Modifier.size(18.dp),
                tint = if (enabled) LocalContentColor.current else LocalContentColor.current.copy(alpha = 0.38f)
            )
        }
    }
}
