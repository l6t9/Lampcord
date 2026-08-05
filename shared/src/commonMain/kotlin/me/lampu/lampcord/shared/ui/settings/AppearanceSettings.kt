package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AppearanceSettings(chatState: ChatState) {
    val scrollState = rememberScrollState()
    
    Column(modifier = Modifier.fillMaxWidth().verticalScroll(scrollState).padding(bottom = 32.dp)) {
        Text("Appearance", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(24.dp))
        
        Text("Theme", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))

        var selectedTheme by remember { mutableStateOf(chatState.userSettings?.theme ?: "Dark") }
        val themes =
            listOf(
                "Dark" to Icons.Filled.Bedtime,
                "Light" to Icons.Filled.LightMode,
                "AMOLED" to Icons.Filled.DarkMode,
            )

        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
            themes.forEachIndexed { index, (label, icon) ->
                SegmentedButton(
                    selected = selectedTheme.equals(label, ignoreCase = true),
                    onClick = { 
                        selectedTheme = label
                        chatState.userSettings = chatState.userSettings?.copy(theme = label.lowercase())
                    },
                    shape = SegmentedButtonDefaults.itemShape(index = index, count = themes.size),
                    icon = { Icon(icon, contentDescription = null, modifier = Modifier.size(18.dp)) }
                ) {
                    Text(label)
                }
            }
        }

        Spacer(modifier = Modifier.height(32.dp))
        Text("Color", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))
        
        // Simulating Material You color selection
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
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

        val devMode = chatState.userSettings?.developer_mode ?: false
        Row(
            modifier = Modifier.fillMaxWidth().clickable { 
                chatState.userSettings = chatState.userSettings?.copy(developer_mode = !devMode)
            },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("Developer Mode", style = MaterialTheme.typography.bodyLarge)
                Text("Exposes ID copying and other advanced debug tools.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            ExpressiveSwitch(checked = devMode, onCheckedChange = { 
                chatState.userSettings = chatState.userSettings?.copy(developer_mode = it)
            })
        }
        
        Spacer(modifier = Modifier.height(24.dp))
        
        var compactMode by remember { mutableStateOf(false) }
        Row(
            modifier = Modifier.fillMaxWidth().clickable { compactMode = !compactMode },
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text("Compact Messages", style = MaterialTheme.typography.bodyLarge)
                Text("Use a denser layout for chat messages.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            ExpressiveSwitch(checked = compactMode, onCheckedChange = { compactMode = it })
        }
    }
}
