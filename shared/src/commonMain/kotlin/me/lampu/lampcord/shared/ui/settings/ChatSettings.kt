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
import me.lampu.lampcord.shared.settings.ChatGestures
import me.lampu.lampcord.shared.settings.Settings

@Composable
fun ChatSettings(chatState: ChatState) {
    val userSettings = chatState.userSettings
    var gesturesExpanded by remember { mutableStateOf(false) }
    var nitroExpanded by remember { mutableStateOf(false) }
    var loggerExpanded by remember { mutableStateOf(false) }
    
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Gestures",
            items = listOf(
                switchSettingsItem(
                    title = "TapTap",
                    description = "Double tap a message to edit or reply.",
                    checked = Settings.shared.tapTap,
                    onCheckedChange = { 
                        Settings.shared.tapTap = it
                    }
                ),
                expandableSettingsItem(
                    title = "Swipe Gesture",
                    description = when(Settings.shared.chatGestures) {
                        ChatGestures.SWIPE_TO_MEMBERS -> "Swipe to view member list"
                        ChatGestures.SWIPE_TO_REPLY -> "Swipe to reply"
                    },
                    expanded = gesturesExpanded,
                    onToggle = { gesturesExpanded = !gesturesExpanded }
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        ChatGestures.entries.forEach { gesture ->
                            val label = when(gesture) {
                                ChatGestures.SWIPE_TO_REPLY -> "Swipe to reply"
                                ChatGestures.SWIPE_TO_MEMBERS -> "Swipe to view member list"
                            }
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clickable { 
                                        Settings.shared.chatGestures = gesture
                                        gesturesExpanded = false
                                    }
                                    .padding(12.dp),
                                verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                            ) {
                                RadioButton(selected = Settings.shared.chatGestures == gesture, onClick = null)
                                Spacer(Modifier.width(8.dp))
                                Text(label)
                            }
                        }
                    }
                }
            )
        )

        Material3SettingsGroup(
            title = "Display images, videos, and lolcats",
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

        Material3SettingsGroup(
            title = "Embeds and Link Previews",
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

        Material3SettingsGroup(
            title = "Emoji",
            items = listOf(
                switchSettingsItem(
                    title = "Animate Emoji",
                    checked = userSettings?.animate_emoji ?: true,
                    onCheckedChange = { 
                        chatState.updateUserSettings(UserSettings.Partial(animate_emoji = it))
                    }
                ),
                expandableSettingsItem(
                    title = "Free Nitro Emojis",
                    description = "Use emojis from any server for free.",
                    expanded = nitroExpanded,
                    onToggle = { nitroExpanded = !nitroExpanded }
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        switchSettingsItem(
                            title = "Enable Free Nitro Emojis",
                            checked = Settings.shared.freeNitroEmojis,
                            onCheckedChange = { Settings.shared.freeNitroEmojis = it }
                        ).let { Material3SettingsItemRow(it, isFirst = true, horizontalPadding = 0.dp) }
                        
                        switchSettingsItem(
                            title = "Enable Realmojis",
                            description = "Makes the client think free nitro emojis are real nitro emojis.",
                            checked = Settings.shared.realmojis,
                            onCheckedChange = { Settings.shared.realmojis = it }
                        ).let { Material3SettingsItemRow(it, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Realmojis in compound sentences",
                            description = "Allows messages like 'hello :emoji: world' to display properly.",
                            checked = Settings.shared.compoundRealmojis,
                            onCheckedChange = { Settings.shared.compoundRealmojis = it }
                        ).let { Material3SettingsItemRow(it, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Use WebP format",
                            description = "Use WebP for all emojis instead of GIF/PNG.",
                            checked = Settings.shared.useWebpEmojis,
                            onCheckedChange = { Settings.shared.useWebpEmojis = it }
                        ).let { Material3SettingsItemRow(it, isLast = true, horizontalPadding = 0.dp) }
                    }
                }
            )
        )

        var stickersExpanded by remember { mutableStateOf(false) }
        Material3SettingsGroup(
            title = "Stickers",
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

        Material3SettingsGroup(
            title = "Enhancements",
            items = listOf(
                switchSettingsItem(
                    title = "Bypass Upload Limit",
                    description = "Ignore client-side file size warnings.",
                    checked = Settings.shared.bypassUploadLimit,
                    onCheckedChange = { Settings.shared.bypassUploadLimit = it }
                ),
                expandableSettingsItem(
                    title = "Message Logger",
                    description = "Keep a local history of deleted and edited messages.",
                    expanded = loggerExpanded,
                    onToggle = { loggerExpanded = !loggerExpanded }
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        switchSettingsItem(
                            title = "Enable Message Logger",
                            checked = Settings.shared.messageLoggerEnabled,
                            onCheckedChange = { Settings.shared.messageLoggerEnabled = it }
                        ).let { Material3SettingsItemRow(it, isFirst = true, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Ignore Bots",
                            checked = Settings.shared.messageLoggerIgnoreBots,
                            onCheckedChange = { Settings.shared.messageLoggerIgnoreBots = it }
                        ).let { Material3SettingsItemRow(it, horizontalPadding = 0.dp) }

                        switchSettingsItem(
                            title = "Ignore Self",
                            checked = Settings.shared.messageLoggerIgnoreSelf,
                            onCheckedChange = { Settings.shared.messageLoggerIgnoreSelf = it }
                        ).let { Material3SettingsItemRow(it, isLast = true, horizontalPadding = 0.dp) }
                    }
                }
            )
        )
    }
}
