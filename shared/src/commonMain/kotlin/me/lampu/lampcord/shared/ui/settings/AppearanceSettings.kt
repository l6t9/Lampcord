package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*

@Composable
fun AppearanceSettings(chatState: ChatState) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(title = "Theme") {
            Column(modifier = Modifier.padding(horizontal = 16.dp)) {
                val themes = listOf(
                    "Auto" to Icons.Filled.Public,
                    "Dark" to Icons.Filled.Bedtime,
                    "Light" to Icons.Filled.LightMode,
                    "AMOLED" to Icons.Filled.DarkMode,
                )

                SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                    themes.forEachIndexed { index, (label, icon) ->
                        val themeValue = label.lowercase()
                        val isSelected = chatState.settingsStore.themeMode == themeValue

                        SegmentedButton(
                            selected = isSelected,
                            onClick = { 
                                chatState.settingsStore.themeMode = themeValue
                                if (themeValue == "amoled") {
                                    chatState.settingsStore.pureBlack = true
                                } else if (themeValue != "auto") {
                                    chatState.settingsStore.pureBlack = false
                                }
                            },
                            shape = SegmentedButtonDefaults.itemShape(index = index, count = themes.size),
                            icon = { Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp)) }
                        ) {
                            Text(label)
                        }
                    }
                }
            }
            
            Spacer(Modifier.height(12.dp))

            if (chatState.settingsStore.themeMode == "auto") {
                Material3SettingsGroup(
                    items = listOf(
                        switchSettingsItem(
                            title = "Pure Black in Dark Mode",
                            description = "Use pure black backgrounds when the system is in dark mode.",
                            checked = chatState.settingsStore.pureBlack,
                            onCheckedChange = { chatState.settingsStore.pureBlack = it }
                        )
                    ),
                    horizontalPadding = 0.dp
                )
            }
        }

        Material3SettingsGroup(title = "Color") {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                val colors = listOf(
                    Color(0xFF6750A4), // Baseline
                    Color(0xFF984061), // Rose
                    Color(0xFF216D2F), // Green
                    Color(0xFF0061A4), // Blue
                    Color(0xFF6B5E00)  // Yellow
                )
                
                colors.forEach { color ->
                    Box(
                        modifier = Modifier
                            .size(48.dp)
                            .clip(CircleShape)
                            .background(color)
                            .clickable { /* TODO: Change app accent color */ }
                    )
                }
            }
        }

        Material3SettingsGroup(title = "Display") {
            var showNitro by remember { mutableStateOf(true) }
            var compactMode by remember { mutableStateOf(false) }

            Material3SettingsGroup(
                items = listOf(
                    switchSettingsItem(
                        title = "Show Nitro Badge",
                        description = "Display the Nitro badge on your profile if you have an active subscription.",
                        checked = showNitro,
                        onCheckedChange = { showNitro = it }
                    ),
                    switchSettingsItem(
                        title = "Compact Messages",
                        description = "Use a denser layout for chat messages.",
                        checked = compactMode,
                        onCheckedChange = { compactMode = it }
                    )
                ),
                horizontalPadding = 0.dp
            )
        }
    }
}
