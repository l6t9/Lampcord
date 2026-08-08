package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.model.UserSettings

@Composable
fun AccessibilitySettings(chatState: ChatState) {
    val userSettings = chatState.userSettings

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
