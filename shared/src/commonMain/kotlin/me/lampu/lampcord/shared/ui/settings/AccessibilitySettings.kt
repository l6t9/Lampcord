package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.settings.Settings
import androidx.compose.ui.text.font.FontWeight
import org.koin.compose.koinInject

@Composable
fun AccessibilitySettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Accessibility",
        onNavigateBack = onBack
    ) {
        AccessibilitySettingsContent(settingsStore = settingsStore)
    }
}

@Composable
fun AccessibilitySettingsContent(settingsStore: SettingsStore = koinInject()) {
    val userSettings = settingsStore.userSettings
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Visual",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Saturation") },
                    description = {
                        var saturation by remember { mutableStateOf(1f) }
                        Column(modifier = Modifier.padding(top = 8.dp)) {
                            Text("${(saturation * 100).toInt()}%", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
                            Slider(
                                value = saturation,
                                onValueChange = { saturation = it },
                                valueRange = 0f..1f,
                                modifier = Modifier.fillMaxWidth()
                            )
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Motion & Contrast",
            items = listOf(
                switchSettingsItem(
                    title = "Reduced Motion",
                    description = "Reduces the amount of animation and movement in the UI.",
                    checked = Settings.shared.reduceMotion,
                    onCheckedChange = { Settings.shared.reduceMotion = it }
                ),
                switchSettingsItem(
                    title = "High Contrast",
                    description = "Increases contrast between foreground and background elements.",
                    checked = false, // TODO
                    onCheckedChange = { }
                )
            )
        )

        Material3SettingsGroup(
            title = "Detection",
            items = listOf(
                switchSettingsItem(
                    title = "Allow Accessibility Detection",
                    description = "Allow Discord to detect if you are using accessibility tools.",
                    checked = userSettings?.allow_accessibility_detection ?: false,
                    onCheckedChange = { 
                        settingsStore.updateUserSettings(UserSettings.Partial(allow_accessibility_detection = it))
                    }
                )
            )
        )
    }
}
