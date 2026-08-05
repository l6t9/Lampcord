package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch

@Composable
fun AccessibilitySettings(chatState: ChatState) {
    Column {
        Text("Accessibility", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(24.dp))

        var saturation by remember { mutableStateOf(1f) }
        Text("Saturation: ${(saturation * 100).toInt()}%", style = MaterialTheme.typography.titleSmall)
        Slider(
            value = saturation,
            onValueChange = { saturation = it },
            valueRange = 0f..1f,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(24.dp))

        var reducedMotion by remember { mutableStateOf(false) }
        Row(
            modifier = Modifier.fillMaxWidth().clickable { reducedMotion = !reducedMotion },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("Reduced Motion", style = MaterialTheme.typography.bodyLarge)
                Text("Reduces the amount of animation and movement in the UI.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            ExpressiveSwitch(checked = reducedMotion, onCheckedChange = { reducedMotion = it })
        }
        
        Spacer(modifier = Modifier.height(24.dp))
        
        var highContrast by remember { mutableStateOf(false) }
        Row(
            modifier = Modifier.fillMaxWidth().clickable { highContrast = !highContrast },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("High Contrast", style = MaterialTheme.typography.bodyLarge)
                Text("Increases contrast between foreground and background elements.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            ExpressiveSwitch(checked = highContrast, onCheckedChange = { highContrast = it })
        }
    }
}
