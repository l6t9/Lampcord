package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.guilds.GuildChannelList
import me.lampu.lampcord.shared.ui.components.guilds.GuildRail

@Composable
fun Sidebar(chatState: ChatState, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier
            .fillMaxHeight(),
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Row(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            GuildRail(chatState)

            // Channels / DMs List
            Surface(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                shape = MaterialTheme.shapes.large,
                tonalElevation = 1.dp
            ) {
                if (chatState.selectedGuild != null) {
                    GuildChannelList(chatState)
                } else {
                    DMList(chatState)
                }
            }
        }

        if (chatState.isVoiceConnected) {
            VoiceConnectionPanel(chatState)
        }

        // Account Panel (CurrentUser)
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .height(60.dp),
            shape = MaterialTheme.shapes.medium,
            tonalElevation = 1.dp
        ) {
            AccountPanel(chatState)
        }
    }
}
