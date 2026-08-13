package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.settings.ChatGestures
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import org.koin.compose.koinInject

@Composable
fun ChatSettings(settingsStore: SettingsStore = koinInject()) {
    val userSettings = settingsStore.userSettings
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopChatSettings(settingsStore, userSettings)
    } else {
        MobileChatSettings(settingsStore, userSettings)
    }
}

@Composable
private fun DesktopChatSettings(settingsStore: SettingsStore, userSettings: UserSettings?) {
    SettingsLayout {
        SettingsSection(
            title = "Display",
            icon = Icons.Rounded.Forum
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                ChatToggle("Chat Bubbles", settingsStore.chatBubbles, "Display messages inside rounded chat bubbles.") {
                    settingsStore.chatBubbles = it
                }
            }
        }

        SettingsSection(
            title = "Gestures",
            icon = Icons.Filled.DragIndicator
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                ChatToggle("TapTap", Settings.shared.tapTap, "Double tap a message to edit or reply.") {
                    Settings.shared.tapTap = it
                }

                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Swipe Gesture", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    SettingsButtonGroup(
                        options = ChatGestures.entries.toList(),
                        selectedOption = Settings.shared.chatGestures,
                        onOptionSelected = { Settings.shared.chatGestures = it },
                        iconProvider = { gesture: ChatGestures, isSelected ->
                            when (gesture) {
                                ChatGestures.SWIPE_TO_MEMBERS -> if (isSelected) Icons.Filled.Group else Icons.Rounded.Group
                                ChatGestures.SWIPE_TO_REPLY -> if (isSelected) Icons.Filled.Reply else Icons.Rounded.Reply
                            }
                        },
                        labelProvider = {
                            when (it) {
                                ChatGestures.SWIPE_TO_MEMBERS -> "View members"
                                ChatGestures.SWIPE_TO_REPLY -> "Reply"
                            }
                        }
                    )
                }
            }
        }

        SettingsSection(
            title = "Media",
            icon = Icons.Filled.Album
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                ChatToggle("Auto-display uploads", userSettings?.inline_attachment_media ?: true, "Images and videos uploaded directly to Discord.") {
                    settingsStore.updateUserSettings(UserSettings.Partial(inline_attachment_media = it))
                }
                ChatToggle("Auto-display links", userSettings?.inline_embed_media ?: true, "Links to rich media from other websites.") {
                    settingsStore.updateUserSettings(UserSettings.Partial(inline_embed_media = it))
                }
                ChatToggle("Show embeds", userSettings?.render_embeds ?: true, "Previews for website links pasted into chat.") {
                    settingsStore.updateUserSettings(UserSettings.Partial(render_embeds = it))
                }
            }
        }

        SettingsSection(
            title = "Emoji and Stickers",
            icon = Icons.Filled.Mood
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                ChatToggle("Animate Emoji", userSettings?.animate_emoji ?: true) {
                    settingsStore.updateUserSettings(UserSettings.Partial(animate_emoji = it))
                }

                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("Animate Stickers", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface)
                    SettingsButtonGroup(
                        options = listOf(0, 1, 2),
                        selectedOption = userSettings?.animate_stickers ?: 0,
                        onOptionSelected = { settingsStore.updateUserSettings(UserSettings.Partial(animate_stickers = it)) },
                        iconProvider = { level: Int, isSelected ->
                            when (level) {
                                0 -> if (isSelected) Icons.Filled.PlayArrow else Icons.Rounded.PlayArrow
                                1 -> if (isSelected) Icons.Filled.AddReaction else Icons.Rounded.AddReaction
                                else -> if (isSelected) Icons.Filled.Pause else Icons.Rounded.Pause
                            }
                        },
                        labelProvider = {
                            when (it) {
                                0 -> "Always"
                                1 -> "On interaction"
                                else -> "Never"
                            }
                        }
                    )
                }

                ChatToggle("Free Nitro Emojis", Settings.shared.freeNitroEmojis, "Use emojis from any server for free.") {
                    Settings.shared.freeNitroEmojis = it
                }
                ChatToggle("Realmojis", Settings.shared.realmojis, "Makes free nitro emojis look like real ones.") {
                    Settings.shared.realmojis = it
                }
            }
        }

        SettingsSection(
            title = "Logger",
            icon = Icons.Filled.History
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                ChatToggle("Message Logger", Settings.shared.messageLoggerEnabled, "Keep a local history of deleted and edited messages.") {
                    Settings.shared.messageLoggerEnabled = it
                }
                if (Settings.shared.messageLoggerEnabled) {
                    ChatToggle("Ignore Bots", Settings.shared.messageLoggerIgnoreBots) {
                        Settings.shared.messageLoggerIgnoreBots = it
                    }
                    ChatToggle("Ignore Self", Settings.shared.messageLoggerIgnoreSelf) {
                        Settings.shared.messageLoggerIgnoreSelf = it
                    }
                }
            }
        }
    }
}

@Composable
private fun ChatToggle(label: String, checked: Boolean, description: String? = null, onCheckedChange: (Boolean) -> Unit) {
    Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
        Column(modifier = Modifier.weight(1f)) {
            Text(label, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            if (description != null) {
                Text(description, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        ExpressiveSwitch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun MobileChatSettings(settingsStore: SettingsStore, userSettings: UserSettings?) {
    var gesturesExpanded by remember { mutableStateOf(false) }
    var nitroExpanded by remember { mutableStateOf(false) }
    var loggerExpanded by remember { mutableStateOf(false) }
    
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Display",
            items = listOf(
                switchSettingsItem(
                    title = "Chat Bubbles",
                    description = "Display messages inside rounded chat bubbles.",
                    checked = settingsStore.chatBubbles,
                    onCheckedChange = {
                        settingsStore.chatBubbles = it
                    }
                )
            )
        )

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
                        settingsStore.updateUserSettings(UserSettings.Partial(inline_attachment_media = it))
                    }
                ),
                switchSettingsItem(
                    title = "When linked from websites",
                    description = "Links to images and videos will be automatically converted to rich media.",
                    checked = userSettings?.inline_embed_media ?: true,
                    onCheckedChange = { 
                        settingsStore.updateUserSettings(UserSettings.Partial(inline_embed_media = it))
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
                        settingsStore.updateUserSettings(UserSettings.Partial(render_embeds = it))
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
                        settingsStore.updateUserSettings(UserSettings.Partial(animate_emoji = it))
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
                                        settingsStore.updateUserSettings(UserSettings.Partial(animate_stickers = index))
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
