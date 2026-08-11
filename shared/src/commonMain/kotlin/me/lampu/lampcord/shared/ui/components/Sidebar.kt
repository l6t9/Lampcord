package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.components.guilds.GuildChannelList
import me.lampu.lampcord.shared.ui.components.guilds.GuildRail
import org.koin.compose.koinInject

@Composable
fun Sidebar(
    navigationStore: NavigationStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxHeight(),
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Row(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            GuildRail()

            // Channels / DMs List
            Surface(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                shape = MaterialTheme.shapes.large,
                tonalElevation = 1.dp
            ) {
                if (navigationStore.selectedGuild != null) {
                    GuildChannelList()
                } else {
                    DMList()
                }
            }
        }

        if (voiceStore.isVoiceConnected) {
            VoiceConnectionPanel()
        }

        // Account Panel (CurrentUser)
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .height(60.dp),
            shape = MaterialTheme.shapes.medium,
            tonalElevation = 1.dp
        ) {
            AccountPanel()
        }
    }
}
