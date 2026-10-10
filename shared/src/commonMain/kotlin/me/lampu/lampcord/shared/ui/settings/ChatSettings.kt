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
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.settings.ChatGestures
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.TapTapAction
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun ChatSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Chat",
        onNavigateBack = onBack
    ) {
        ChatSettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun ChatSettingsContent(
    settingsStore: SettingsStore = koinInject()
) {
    val userSettings = settingsStore.userSettings
    val platform = remember { getPlatformName() }
    val isDesktop = platform != "android" && platform != "ios"
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
                ),
                switchSettingsItem(
                    title = "Show Call Button",
                    description = "Show the call button in DM headers.",
                    checked = settingsStore.showCallButton,
                    onCheckedChange = { settingsStore.showCallButton = it }
                )
            )
        )

        Material3SettingsGroup(
            title = "Voice",
            items = listOf(
                switchSettingsItem(
                    title = "Noise Cancellation",
                    description = "Remove background noise from your microphone using RNNoise.",
                    checked = settingsStore.noiseCancellation,
                    onCheckedChange = { settingsStore.noiseCancellation = it }
                )
            )
        )

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
                                        TapTapAction.DELETE_MESSAGE -> if (isSelected) Icons.Filled.Delete else Icons.Rounded.Delete
                                        TapTapAction.DISABLED -> if (isSelected) Icons.Filled.Close else Icons.Rounded.Close
                                    }
                                },
                                labelProvider = {
                                    when (it) {
                                        TapTapAction.REPLY_OR_EDIT -> "Reply/Edit"
                                        TapTapAction.EMOJI_PICKER -> "React"
                                        TapTapAction.DELETE_MESSAGE -> "Delete"
                                        TapTapAction.DISABLED -> "Disabled"
                                    }
                                }
                            )
                        }
                    }
                ))
                add(switchSettingsItem(
                    title = "Swipe to Reply",
                    description = "Swipe a message right to reply to it.",
                    checked = Settings.shared.swipeToReplyEnabled,
                    onCheckedChange = { Settings.shared.swipeToReplyEnabled = it }
                ))
            }
        )

        if (!isDesktop) {
            Material3SettingsGroup(
                title = "Mobile Layout",
                items = listOf(
                    switchSettingsItem(
                        title = "Show Channel List Button",
                        description = "Display the button that opens the channel list in the chat header.",
                        checked = settingsStore.mobileShowChannelListButton,
                        onCheckedChange = { settingsStore.mobileShowChannelListButton = it }
                    ),
                    switchSettingsItem(
                        title = "Show Member List Button",
                        description = "Display the button that opens the member list in the chat header.",
                        checked = settingsStore.mobileShowMemberListButton,
                        onCheckedChange = { settingsStore.mobileShowMemberListButton = it }
                    )
                )
            )
        }

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
