package com.example.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.ui.components.ExpressiveSwitch
import com.example.lampcord.shared.ui.icons.Icons

@Composable
fun AppearanceSettings(chatState: ChatState) {
    Column {
        Text("Theme", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))

        var selectedTheme by remember { mutableStateOf("Dark") }
        val themes =
            listOf(
                "Dark" to Icons.Filled.Bedtime,
                "Light" to Icons.Filled.LightMode,
                "AMOLED" to Icons.Filled.DarkMode,
            )

        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
            themes.forEachIndexed { index, (label, icon) ->
                SegmentedButton(
                    selected = selectedTheme == label,
                    onClick = { selectedTheme = label },
                    shape = SegmentedButtonDefaults.itemShape(index = index, count = themes.size),
                    icon = { Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp)) }
                ) {
                    Text(label)
                }
            }
        }

        Spacer(modifier = Modifier.height(32.dp))
        Text("Display", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))

        var showNitro by remember { mutableStateOf(true) }
        Row(
            modifier = Modifier.fillMaxWidth().clickable { showNitro = !showNitro },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("Show Nitro Badge", style = MaterialTheme.typography.bodyLarge)
                Text("Display the Nitro badge on your profile if you have an active subscription.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            ExpressiveSwitch(checked = showNitro, onCheckedChange = { showNitro = it })
        }

        Spacer(modifier = Modifier.height(24.dp))

        var animateAvatars by remember { mutableStateOf(true) }
        Row(
            modifier = Modifier.fillMaxWidth().clickable { animateAvatars = !animateAvatars },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("Animate Avatars", style = MaterialTheme.typography.bodyLarge)
                Text("Play animated avatars when hovering over a user.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            ExpressiveSwitch(checked = animateAvatars, onCheckedChange = { animateAvatars = it })
        }
    }
}
