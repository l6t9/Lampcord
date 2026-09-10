package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.MessageLogger
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.settings.ChatGestures
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.TapTapAction
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.showToast
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.ui.settings.TextReplaceSettings
import org.koin.compose.koinInject

@Composable
fun ChatSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    var currentSubTab by remember { mutableStateOf("main") }

    SettingsSubScreen(
        title = if (currentSubTab == "text_replace") "Text Replacement" else "Chat",
        onNavigateBack = {
            if (currentSubTab == "text_replace") {
                currentSubTab = "main"
            } else {
                onBack()
            }
        }
    ) {
        if (currentSubTab == "text_replace") {
            TextReplaceSettings(settingsStore = settingsStore)
        } else {
            ChatSettingsContent(
                settingsStore = settingsStore,
                onNavigateToTextReplace = { currentSubTab = "text_replace" }
            )
        }
    }
}

@Composable
fun ChatSettingsContent(
    settingsStore: SettingsStore = koinInject(),
    messageLogger: MessageLogger = koinInject(),
    onNavigateToTextReplace: () -> Unit = {}
) {
    val userSettings = settingsStore.userSettings
    val platform = remember { getPlatformName() }
    val isDesktop = platform != "android" && platform != "ios"
    var showClearLoggerConfirmation by remember { mutableStateOf(false) }
    var currentSubTab by remember { mutableStateOf("main") }

    if (currentSubTab == "text_replace") {
        SettingsSubScreen(title = "Text Replacement", onNavigateBack = { currentSubTab = "main" }) {
            TextReplaceSettings(settingsStore = settingsStore)
        }
        return
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Display",
            items = listOf(
                switchSettingsItem(
                    title = "Chat Bubbles",
                    description = "Display messages inside rounded chat bubbles.",
                    checked = settingsStore.chatBubbles,
                    onCheckedChange = { settingsStore.chatBubbles = it }
                ),
                switchSettingsItem(
                    title = "Compact Mode",
                    description = "Display messages in a compact IRC-style layout.",
                    checked = settingsStore.compactMode,
                    onCheckedChange = { settingsStore.compactMode = it }
                ),
                switchSettingsItem(
                    title = "Show Message in Context Menu",
                    description = "View the message and quick reactions inside the context menu.",
                    checked = settingsStore.showContextMenuMessage,
                    onCheckedChange = { settingsStore.showContextMenuMessage = it }
                ),
                switchSettingsItem(
                    title = "Show Search in Header",
                    description = "Show the search button in the chat header.",
                    checked = settingsStore.showChatSearch,
                    onCheckedChange = { settingsStore.showChatSearch = it }
                ),
                switchSettingsItem(
                    title = "Show Pins in Header",
                    description = "Show the pinned messages button in the chat header.",
                    checked = settingsStore.showChatPins,
                    onCheckedChange = { settingsStore.showChatPins = it }
                )
            )
        )

        if (isDesktop) {
            Material3SettingsGroup(
                title = "Performance",
                items = listOf(
                    switchSettingsItem(
                        title = "Reduce RAM Usage",
                        description = "Use less memory for images and video on desktop. Changes apply immediately.",
                        checked = settingsStore.desktopLowMemoryMode,
                        onCheckedChange = { settingsStore.desktopLowMemoryMode = it }
                    )
                )
            )
        }

        Material3SettingsGroup(
            title = "Chatbox Customization",
            items = listOf(
                sliderSettingsItem(
                    title = "Font Scale",
                    label = "${(settingsStore.chatboxFontSize * 100).toInt()}%",
                    value = settingsStore.chatboxFontSize,
                    onValueChange = { settingsStore.chatboxFontSize = it },
                    valueRange = 0.5f..2.0f
                ),
                sliderSettingsItem(
                    title = "Background Opacity",
                    label = "${(settingsStore.chatboxBackgroundOpacity * 100).toInt()}%",
                    value = settingsStore.chatboxBackgroundOpacity,
                    onValueChange = { settingsStore.chatboxBackgroundOpacity = it },
                    valueRange = 0.0f..1.0f
                ),
                sliderSettingsItem(
                    title = "Border Radius",
                    label = "${settingsStore.chatboxBorderRadius}dp",
                    value = settingsStore.chatboxBorderRadius.toFloat(),
                    onValueChange = { settingsStore.chatboxBorderRadius = it.toInt() },
                    valueRange = 0.0f..32.0f
                ),
                switchSettingsItem(
                    title = "Hide Upload Button",
                    checked = settingsStore.chatboxHideUploadButton,
                    onCheckedChange = { settingsStore.chatboxHideUploadButton = it }
                ),
                switchSettingsItem(
                    title = "Hide Emoji Button",
                    checked = settingsStore.chatboxHideEmojiButton,
                    onCheckedChange = { settingsStore.chatboxHideEmojiButton = it }
                ),
                switchSettingsItem(
                    title = "Show Avatar in Chatbox",
                    description = "Displays your current avatar inside the chat input bar.",
                    checked = settingsStore.chatboxShowAvatar,
                    onCheckedChange = { settingsStore.chatboxShowAvatar = it }
                ),
                switchSettingsItem(
                        title = "Silent Typing",
                        description = "Show a keyboard control for hiding your typing indicator.",
                        checked = settingsStore.silentTypingButtonEnabled,
                        onCheckedChange = { settingsStore.silentTypingButtonEnabled = it }
                )
            ).let { items ->
                if (platform == "android") {
                    items.toMutableList().apply {
                        add(
                            5,
                            switchSettingsItem(
                                title = "Hide Voice Message Button",
                                description = "Remove the voice message recording button from the chat box.",
                                checked = settingsStore.chatboxHideVoiceButton,
                                onCheckedChange = { settingsStore.chatboxHideVoiceButton = it }
                            )
                        )
                    }
                } else items
            }
        )

        Material3SettingsGroup(
            title = "Gestures",
            items = buildList {
                add(Material3SettingsItem(
                    title = { Text("TapTap Action") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp)) {
                            SettingsButtonGroup(
                                options = TapTapAction.entries.toList(),
                                selectedOption = Settings.shared.tapTap,
                                onOptionSelected = { Settings.shared.tapTap = it },
                                iconProvider = { gesture: TapTapAction, isSelected ->
                                    when (gesture) {
                                        TapTapAction.REPLY_OR_EDIT -> if (isSelected) Icons.Filled.Reply else Icons.Rounded.Reply
                                        TapTapAction.EMOJI_PICKER -> if (isSelected) Icons.Filled.AddReaction else Icons.Rounded.AddReaction
                                        TapTapAction.DISABLED -> if (isSelected) Icons.Filled.Close else Icons.Rounded.Close
                                    }
                                },
                                labelProvider = {
                                    when (it) {
                                        TapTapAction.REPLY_OR_EDIT -> "Reply/Edit"
                                        TapTapAction.EMOJI_PICKER -> "React"
                                        TapTapAction.DISABLED -> "Disabled"
                                    }
                                }
                            )
                        }
                    }
                ))
                add(Material3SettingsItem(
                    title = { Text("Swipe Gesture") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp)) {
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
                ))
            }
        )

        Material3SettingsGroup(
            title = "Media",
            items = listOf(
                switchSettingsItem(
                    title = "Auto-display uploads",
                    description = "Images and videos uploaded directly to Discord.",
                    checked = userSettings?.inline_attachment_media ?: true,
                    onCheckedChange = { settingsStore.updateUserSettings(UserSettings.Partial(inline_attachment_media = it)) }
                ),
                switchSettingsItem(
                    title = "Auto-display links",
                    description = "Links to rich media from other websites.",
                    checked = userSettings?.inline_embed_media ?: true,
                    onCheckedChange = { settingsStore.updateUserSettings(UserSettings.Partial(inline_embed_media = it)) }
                ),
                switchSettingsItem(
                    title = "Show embeds",
                    description = "Previews for website links pasted into chat.",
                    checked = userSettings?.render_embeds ?: true,
                    onCheckedChange = { settingsStore.updateUserSettings(UserSettings.Partial(render_embeds = it)) }
                )
            )
        )

        Material3SettingsGroup(
            title = "Emoji and Stickers",
            items = buildList {
                add(switchSettingsItem(
                    title = "Animate Emoji",
                    checked = userSettings?.animate_emoji ?: true,
                    onCheckedChange = { settingsStore.updateUserSettings(UserSettings.Partial(animate_emoji = it)) }
                ))
                add(Material3SettingsItem(
                    title = { Text("Animate Stickers") },
                    description = {
                        Column(modifier = Modifier.padding(top = 8.dp)) {
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
                    }
                ))
                add(switchSettingsItem(
                    title = "Free Nitro Emojis",
                    description = "Use emojis from any server for free.",
                    checked = Settings.shared.freeNitroEmojis,
                    onCheckedChange = { Settings.shared.freeNitroEmojis = it }
                ))
                add(switchSettingsItem(
                    title = "Realmojis",
                    description = "Makes free nitro emojis look like real ones.",
                    checked = Settings.shared.realmojis,
                    onCheckedChange = { Settings.shared.realmojis = it }
                ))
            }
        )

        Material3SettingsGroup(
            title = "Logger",
            items = buildList {
                add(switchSettingsItem(
                    title = "Message Logger",
                    description = "Keep a local history of deleted and edited messages.",
                    checked = Settings.shared.messageLoggerEnabled,
                    onCheckedChange = { Settings.shared.messageLoggerEnabled = it }
                ))
                if (Settings.shared.messageLoggerEnabled) {
                    add(switchSettingsItem(
                        title = "Ignore Bots",
                        checked = Settings.shared.messageLoggerIgnoreBots,
                        onCheckedChange = { Settings.shared.messageLoggerIgnoreBots = it }
                    ))
                    add(switchSettingsItem(
                        title = "Ignore Self",
                        checked = Settings.shared.messageLoggerIgnoreSelf,
                        onCheckedChange = { Settings.shared.messageLoggerIgnoreSelf = it }
                    ))
                }
                add(Material3SettingsItem(
                    title = { Text("Clear Logged Messages") },
                    description = { Text("Permanently delete every message saved by Message Logger on this device.") },
                    onClick = { showClearLoggerConfirmation = true }
                ))
            }
        )

        Material3SettingsGroup(
            title = "Text Replacement",
            items = listOf(
                Material3SettingsItem(
                    icon = Icons.Rounded.TextFields,
                    title = { Text("Text Replacement Rules") },
                    description = { Text("Manage automatic text substitution rules.") },
                    onClick = {
                        if (platform != "android" && platform != "ios") {
                            currentSubTab = "text_replace"
                        } else {
                            onNavigateToTextReplace()
                        }
                    }
                )
            )
        )
    }

    if (showClearLoggerConfirmation) {
        AlertDialog(
            onDismissRequest = { showClearLoggerConfirmation = false },
            title = { Text("Clear logged messages?") },
            text = { Text("This permanently deletes all Message Logger records stored on this device. This cannot be undone.") },
            confirmButton = {
                TextButton(onClick = {
                    showClearLoggerConfirmation = false
                    messageLogger.clearLoggedMessages { showToast("Logged messages cleared") }
                }) { Text("Clear") }
            },
            dismissButton = {
                TextButton(onClick = { showClearLoggerConfirmation = false }) { Text("Cancel") }
            }
        )
    }
}

@Composable
private fun sliderSettingsItem(
    title: String,
    value: Float,
    onValueChange: (Float) -> Unit,
    valueRange: ClosedFloatingPointRange<Float> = 0f..1f,
    steps: Int = 0,
    label: String? = null
): Material3SettingsItem {
    return Material3SettingsItem(
        title = {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text(title)
                if (label != null) Text(label, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
            }
        },
        description = {
            Slider(
                value = value,
                onValueChange = onValueChange,
                valueRange = valueRange,
                steps = steps,
                modifier = Modifier.fillMaxWidth().padding(top = 8.dp)
            )
        },
        onClick = null
    )
}
