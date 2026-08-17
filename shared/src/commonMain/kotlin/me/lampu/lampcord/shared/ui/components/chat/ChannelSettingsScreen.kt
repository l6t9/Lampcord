package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.guilds.settings.ChannelEditor
import me.lampu.lampcord.shared.ui.components.settings.SettingsLayout
import me.lampu.lampcord.shared.ui.components.settings.SettingsSection
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChannelSettingsScreen(
    onDismiss: () -> Unit,
    navigationStore: NavigationStore = koinInject(),
    channelApi: ChannelApi = koinInject()
) {
    val channel = navigationStore.channelSettingsChannel ?: return

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp
        val content: @Composable () -> Unit = {
            SettingsSubScreen(
                title = "Channel Settings",
                onNavigateBack = onDismiss
            ) {
                if (channel.type == 4 || channel.type == 2 || channel.type == 13) {
                    SettingsLayout {
                        SettingsSection(title = "Channel Details", icon = Icons.Filled.Info) {
                            Text(
                                text = "Use the server settings to edit this channel type.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        TextButton(onClick = {
                            val guildId = channel.guild_id ?: "@me"
                            setClipboardText("https://discord.com/channels/$guildId/${channel.id}")
                        }) {
                            Text("Copy Channel Link")
                        }
                    }
                } else {
                    ChannelEditor(channel, channelApi, onDone = onDismiss)
                    HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))
                    SettingsLayout {
                        SettingsSection(title = "Notifications", icon = Icons.Filled.Notifications) {
                            TextButton(onClick = {
                                val guildId = channel.guild_id ?: "@me"
                                setClipboardText("https://discord.com/channels/$guildId/${channel.id}")
                            }) {
                                Text("Copy Channel Link")
                            }
                        }
                    }
                }
            }
        }

        if (isCompact) {
            content()
        } else {
            Dialog(
                onDismissRequest = onDismiss,
                properties = DialogProperties(usePlatformDefaultWidth = false)
            ) {
                content()
            }
        }
    }
}
