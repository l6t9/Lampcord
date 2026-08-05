package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch

@Composable
fun AdvancedSettings(chatState: ChatState) {
    val scrollState = rememberScrollState()
    
    Column(modifier = Modifier.fillMaxWidth().verticalScroll(scrollState).padding(bottom = 32.dp)) {
        Text("Advanced", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(24.dp))
        
        val devMode = chatState.userSettings?.developer_mode ?: false
        AdvancedToggle("Developer Mode", "Exposes ID copying and other advanced debug tools.", devMode) {
            chatState.userSettings = chatState.userSettings?.copy(developer_mode = it)
        }

        Spacer(modifier = Modifier.height(32.dp))
        Text("Data", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))
        
        AdvancedAction("Clear Cache", "Delete all locally cached images and messages.") {
            // TODO: Implementation
        }
        
        AdvancedAction("Reset App State", "Restore all settings to their default values.") {
            // TODO: Implementation
        }
    }
}

@Composable
fun AdvancedToggle(title: String, description: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().clickable { onCheckedChange(!checked) }.padding(vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(title, style = MaterialTheme.typography.bodyLarge)
            Text(description, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        ExpressiveSwitch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
fun AdvancedAction(title: String, description: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().clickable { onClick() }.padding(vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(title, style = MaterialTheme.typography.bodyLarge)
            Text(description, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Button(onClick = onClick, colors = ButtonDefaults.filledTonalButtonColors()) {
            Text(title.split(" ").first())
        }
    }
}
