package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
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
    modifier: Modifier = Modifier
) {
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"

    Column(
        modifier = modifier.fillMaxHeight(),
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
                if (navigationStore.selectedGuild != null) {
                    GuildChannelList()
                } else {
                    DMList()
                }
            }
        }

        if (!isMobile) {
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
                        .border(
                            1.dp,
                            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f),
                            RoundedCornerShape(16.dp)
                        )
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
    val guilds by guildStore.guilds.collectAsState()
    val guild = remember(activeChannel.guild_id, guilds) {
        guilds.find { it.id == activeChannel.guild_id }
    }
    
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 4.dp),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        tonalElevation = 2.dp
    ) {
        Column(modifier = Modifier.padding(8.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    Icons.Filled.Campaign, 
                    null, 
                    tint = Color(0xFF23A559),
                    modifier = Modifier.size(20.dp)
                )
                Spacer(Modifier.width(8.dp))
                Column(Modifier.weight(1f)) {
                    Text(
                        "Voice Connected",
                        style = MaterialTheme.typography.labelMedium,
                        color = Color(0xFF23A559),
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        "${activeChannel.name ?: "Voice Call"} / ${guild?.name ?: "DM"}",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                
                IconButton(onClick = { /* TODO: Noise suppression? */ }, modifier = Modifier.size(24.dp)) {
                    Icon(Icons.Filled.Tune, null, modifier = Modifier.size(16.dp))
                }
                
                IconButton(
                    onClick = { voiceStore.disconnectFromVoice() },
                    modifier = Modifier.size(24.dp)
                ) {
                    Icon(Icons.AutoMirrored.Filled.Logout, null, tint = MaterialTheme.colorScheme.error, modifier = Modifier.size(16.dp))
                }
            }
            
            Spacer(Modifier.height(8.dp))
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                VoicePanelButton(Icons.Filled.Videocam, "Video") { /* TODO */ }
                VoicePanelButton(Icons.Filled.ScreenShare, "Screen") { /* TODO */ }
                VoicePanelButton(Icons.Filled.Devices, "Activities") { /* TODO */ }
                VoicePanelButton(Icons.Filled.VolumeUp, "Soundboard") { /* TODO */ }
            }
        }
    }
}

@Composable
private fun VoicePanelButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    contentDescription: String,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        modifier = Modifier.size(width = 44.dp, height = 32.dp),
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHighest
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(icon, contentDescription, modifier = Modifier.size(18.dp))
        }
    }
}
