package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun TextAndImagesSettings(chatState: ChatState) {
    val userSettings = chatState.userSettings
    
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Display images, videos, and lolcats") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "When uploaded directly to Discord",
                        description = "Images and videos will be displayed when they are sent in chat.",
                        checked = userSettings?.inline_attachment_media ?: true,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(inline_attachment_media = it))
                        }
                    ),
                    switchSettingsItem(
                        title = "When linked from websites",
                        description = "Links to images and videos will be automatically converted to rich media.",
                        checked = userSettings?.inline_embed_media ?: true,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(inline_embed_media = it))
                        }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Embeds and Link Previews") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Show embeds and preview website links pasted into chat",
                        checked = userSettings?.render_embeds ?: true,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(render_embeds = it))
                        }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Emoji") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Animate Emoji",
                        checked = userSettings?.animate_emoji ?: true,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(animate_emoji = it))
                        }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Stickers") {
            var stickersExpanded by remember { mutableStateOf(false) }
            Material3SettingsGroup(
                items = listOf(
                    expandableSettingsItem(
                        title = "Animate Stickers",
                        description = when(userSettings?.animate_stickers) {
                            0 -> "Always animate"
                            1 -> "Animate on interaction"
                            2 -> "Never animate"
                            else -> "Always animate"
                        },
                        expanded = stickersExpanded,
                        onToggle = { stickersExpanded = !stickersExpanded }
                    ) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            listOf("Always animate", "Animate on interaction", "Never animate").forEachIndexed { index, label ->
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .clickable { 
                                            chatState.updateUserSettings(UserSettings.Partial(animate_stickers = index))
                                            stickersExpanded = false
                                        }
                                        .padding(12.dp),
                                    verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                                ) {
                                    RadioButton(selected = userSettings?.animate_stickers == index, onClick = null)
                                    Spacer(Modifier.width(8.dp))
                                    Text(label)
                                }
                            }
                        }
                    }
                )
            )
        }
    }
}
