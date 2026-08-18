package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
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
            horizontalArrangement = Arrangement.Start
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

                if (!isMobile) {
                    Spacer(Modifier.height(16.dp))
                    // Account Panel (CurrentUser) spans only ChannelsList
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

        if (voiceStore.isVoiceConnected) {
            Spacer(Modifier.height(16.dp))
            VoiceConnectionPanel()
        }
    }
}
