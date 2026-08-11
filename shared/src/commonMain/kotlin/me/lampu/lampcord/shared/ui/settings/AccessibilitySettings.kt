package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.model.UserSettings

import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.Alignment

@Composable
fun AccessibilitySettings(chatState: ChatState) {
    val userSettings = chatState.userSettings
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopAccessibilitySettings(chatState, userSettings)
    } else {
        MobileAccessibilitySettings(chatState, userSettings)
    }
}

@Composable
private fun DesktopAccessibilitySettings(chatState: ChatState, userSettings: UserSettings?) {
    SettingsLayout {
        SettingsSection(
            title = "Visual",
            icon = Icons.Filled.Visibility
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                var saturation by remember { mutableStateOf(1f) }
                Text("Saturation: ${(saturation * 100).toInt()}%", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Slider(
                    value = saturation,
                    onValueChange = { saturation = it },
                    valueRange = 0f..1f,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }

        SettingsSection(
            title = "Motion & Contrast",
            icon = Icons.Filled.Speed
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                var reducedMotion by remember { mutableStateOf(false) }
                var highContrast by remember { mutableStateOf(false) }

                AccessibilityToggle("Reduced Motion", reducedMotion, "Reduces the amount of animation and movement in the UI.") {
                    reducedMotion = it
                }
                AccessibilityToggle("High Contrast", highContrast, "Increases contrast between foreground and background elements.") {
                    highContrast = it
                }
            }
        }

        SettingsSection(
            title = "Detection",
            icon = Icons.Filled.Accessibility
        ) {
            AccessibilityToggle("Allow Accessibility Detection", userSettings?.allow_accessibility_detection ?: false, "Allow Discord to detect if you are using accessibility tools.") {
                chatState.updateUserSettings(UserSettings.Partial(allow_accessibility_detection = it))
            }
        }
    }
}

@Composable
private fun AccessibilityToggle(label: String, checked: Boolean, description: String? = null, onCheckedChange: (Boolean) -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically) {
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
private fun MobileAccessibilitySettings(chatState: ChatState, userSettings: UserSettings?) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Visual") {
            var saturation by remember { mutableStateOf(1f) }
            
            Material3SettingsGroup(
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Saturation: ${(saturation * 100).toInt()}%") },
                        description = {
                            Slider(
                                value = saturation,
                                onValueChange = { saturation = it },
                                valueRange = 0f..1f,
                                modifier = Modifier.fillMaxWidth()
                            )
                        }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Motion & Contrast") {
            var reducedMotion by remember { mutableStateOf(false) }
            var highContrast by remember { mutableStateOf(false) }

            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Reduced Motion",
                        description = "Reduces the amount of animation and movement in the UI.",
                        checked = reducedMotion,
                        onCheckedChange = { reducedMotion = it }
                    ),
                    switchSettingsItem(
                        title = "High Contrast",
                        description = "Increases contrast between foreground and background elements.",
                        checked = highContrast,
                        onCheckedChange = { highContrast = it }
                    )
                )
            )
        }

        Material3SettingsGroup(title = "Detection") {
            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Allow Accessibility Detection",
                        description = "Allow Discord to detect if you are using a screen reader or other accessibility tools.",
                        checked = userSettings?.allow_accessibility_detection ?: false,
                        onCheckedChange = { 
                            chatState.updateUserSettings(UserSettings.Partial(allow_accessibility_detection = it))
                        }
                    )
                )
            )
        }
    }
}
