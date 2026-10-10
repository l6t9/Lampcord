package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.MessageLogger
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject

@Composable
fun AdditionalSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    var currentSubTab by remember { mutableStateOf("main") }

    SettingsSubScreen(
        title = if (currentSubTab == "text_replace") "Text Replacement" else "Additional",
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
            AdditionalSettingsContent(
                settingsStore = settingsStore,
                onNavigateToTextReplace = { currentSubTab = "text_replace" }
            )
        }
    }
}

@Composable
fun AdditionalSettingsContent(
    settingsStore: SettingsStore = koinInject(),
    messageLogger: MessageLogger = koinInject(),
    onNavigateToTextReplace: () -> Unit = {}
) {
    val platform = remember { getPlatformName() }
    val uriHandler = LocalUriHandler.current
    var showClearLoggerConfirmation by remember { mutableStateOf(false) }
    var show3y3Confirmation by remember { mutableStateOf(false) }
    var showUserBgConfirmation by remember { mutableStateOf(false) }
    var showUserPfpConfirmation by remember { mutableStateOf(false) }
    var currentSubTab by remember { mutableStateOf("main") }

    if (currentSubTab == "text_replace") {
        SettingsSubScreen(title = "Text Replacement", onNavigateBack = { currentSubTab = "main" }) {
            TextReplaceSettings(settingsStore = settingsStore)
        }
        return
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Profile Enhancements",
            items = listOf(
                switchSettingsItem(title = "3y3 Profile Colors", description = "Invisible bio text.", checked = settingsStore.profile3y3, onCheckedChange = { if (it) show3y3Confirmation = true else settingsStore.profile3y3 = false }),
                switchSettingsItem(title = "UserBG Banners", description = "Custom banners database.", checked = settingsStore.userBg, onCheckedChange = { if (it) showUserBgConfirmation = true else settingsStore.userBg = false }),
                switchSettingsItem(title = "UserPFP Icons", description = "Custom icons database.", checked = settingsStore.userPfp, onCheckedChange = { if (it) showUserPfpConfirmation = true else settingsStore.userPfp = false }),
                Material3SettingsItem(icon = Icons.Rounded.Public, title = { Text("Set UserBG Banner") }, description = { Text("Join server.") }, onClick = { uriHandler.openUri("https://discord.gg/ECg96KZ3Fh") }),
            )
        )

        Material3SettingsGroup(
            title = "Chatbox",
            items = listOf(
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
            )
        )

        Material3SettingsGroup(
            title = "Emoji",
            items = listOf(
                switchSettingsItem(
                    title = "Free Nitro Emojis",
                    description = "Use emojis from any server for free.",
                    checked = Settings.shared.freeNitroEmojis,
                    onCheckedChange = { Settings.shared.freeNitroEmojis = it }
                ),
                switchSettingsItem(
                    title = "Realmojis",
                    description = "Makes free nitro emojis look like real ones.",
                    checked = Settings.shared.realmojis,
                    onCheckedChange = { Settings.shared.realmojis = it }
                )
            )
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

    if (show3y3Confirmation) AlertDialog(onDismissRequest = { show3y3Confirmation = false }, title = { Text("Enable 3y3?") }, text = { Text("Embed customizations in bio?") }, confirmButton = { Button(onClick = { settingsStore.profile3y3 = true; show3y3Confirmation = false }) { Text("Enable") } }, dismissButton = { TextButton(onClick = { show3y3Confirmation = false }) { Text("Cancel") } })
    if (showUserBgConfirmation) AlertDialog(onDismissRequest = { showUserBgConfirmation = false }, title = { Text("Enable UserBG?") }, text = { Text("See custom banners?") }, confirmButton = { Button(onClick = { settingsStore.userBg = true; showUserBgConfirmation = false }) { Text("Enable") } }, dismissButton = { TextButton(onClick = { showUserBgConfirmation = false }) { Text("Cancel") } })
    if (showUserPfpConfirmation) AlertDialog(onDismissRequest = { showUserPfpConfirmation = false }, title = { Text("Enable UserPFP?") }, text = { Text("See custom icons?") }, confirmButton = { Button(onClick = { settingsStore.userPfp = true; showUserPfpConfirmation = false }) { Text("Enable") } }, dismissButton = { TextButton(onClick = { showUserPfpConfirmation = false }) { Text("Cancel") } })

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
