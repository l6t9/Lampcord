package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import me.lampu.lampcord.shared.state.ChatState
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
                        onCheckedChange = { /* TODO */ }
                    ),
                    switchSettingsItem(
                        title = "When linked from websites",
                        description = "Links to images and videos will be automatically converted to rich media.",
                        checked = userSettings?.inline_embed_media ?: true,
                        onCheckedChange = { /* TODO */ }
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
                        onCheckedChange = { /* TODO */ }
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
                        onCheckedChange = { /* TODO */ }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Stickers") {
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Animate Stickers") },
                        description = { 
                            Text(when(userSettings?.animate_stickers) {
                                0 -> "Always animate"
                                1 -> "Animate on interaction"
                                2 -> "Never animate"
                                else -> "Always animate"
                            })
                        },
                        onClick = { /* TODO */ }
                    )
                )
            )
        }
    }
}
