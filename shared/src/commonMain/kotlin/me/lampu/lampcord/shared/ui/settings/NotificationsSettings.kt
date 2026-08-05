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
fun NotificationsSettings(chatState: ChatState) {
    val scrollState = rememberScrollState()
    
    Column(modifier = Modifier.fillMaxWidth().verticalScroll(scrollState).padding(bottom = 32.dp)) {
        Text("Notifications", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(24.dp))
        
        Text("Desktop Notifications", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))
        
        var enableDesktop by remember { mutableStateOf(true) }
        NotificationToggle("Enable Desktop Notifications", "Receive push notifications on your device.", enableDesktop) { enableDesktop = it }
        
        var showPreview by remember { mutableStateOf(true) }
        NotificationToggle("Show Message Preview", "Include message content in notifications.", showPreview) { showPreview = it }

        Spacer(modifier = Modifier.height(32.dp))
        Text("Sounds", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        Spacer(modifier = Modifier.height(16.dp))
        
        var messageSound by remember { mutableStateOf(true) }
        NotificationToggle("Message Sound", "Play a sound when you receive a message.", messageSound) { messageSound = it }
        
        var callSound by remember { mutableStateOf(true) }
        NotificationToggle("Incoming Call Sound", "Play a sound when you are being called.", callSound) { callSound = it }
    }
}

@Composable
fun NotificationToggle(title: String, description: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
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
