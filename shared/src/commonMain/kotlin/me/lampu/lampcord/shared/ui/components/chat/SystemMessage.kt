package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.theme.DiscordGreen
import me.lampu.lampcord.shared.ui.theme.DiscordRed
import me.lampu.lampcord.shared.ui.theme.Fuchsia

@Composable
fun SystemMessage(message: Message, chatState: ChatState) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp, horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        val (icon, iconTint, text) = when (message.type) {
            1 -> Triple(Icons.Filled.PersonAdd, DiscordGreen, "${message.author.username} added a recipient.")
            2 -> Triple(Icons.Filled.PersonRemove, DiscordRed, "${message.author.username} removed a recipient.")
            6 -> Triple(Icons.Filled.PushPin, MaterialTheme.colorScheme.primary, "${message.author.username} pinned a message to this channel.")
            7 -> Triple(Icons.AutoMirrored.Filled.ArrowForward, DiscordGreen, "${message.author.global_name ?: message.author.username} joined the server.")
            8, 9, 10, 11 -> Triple(Icons.Filled.RocketLaunch, Fuchsia, "${message.author.global_name ?: message.author.username} just boosted the server!")
            18 -> Triple(Icons.Filled.Tag, MaterialTheme.colorScheme.primary, "${message.author.global_name ?: message.author.username} started a thread.")
            else -> Triple(Icons.Filled.Info, MaterialTheme.colorScheme.onSurfaceVariant, "System message (Type ${message.type})")
        }

        if (message.type == 7) {
            Box(modifier = Modifier.width(44.dp), contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = iconTint
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))

            val randomMessages = listOf("pizzaPre", "slid", "everyoneWelcomePre", "showedUp", "hopped")
            val selectedMessage = randomMessages[message.id.takeLast(1).toInt() % randomMessages.size]

            if (selectedMessage.contains("Pre")) {
                Text(
                    text = when (selectedMessage) {
                        "pizzaPre" -> "Welcome,"
                        "everyoneWelcomePre" -> "Everyone welcome"
                        else -> ""
                    },
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface
                )
                Spacer(modifier = Modifier.width(4.dp))
            }
            
            var profilePosition by remember { mutableStateOf(Offset.Zero) }
            Text(
                text = message.author.global_name ?: message.author.username,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
                modifier = Modifier
                    .onGloballyPositioned { profilePosition = it.positionInRoot() }
                    .clickable { chatState.showProfile(message.author.id, profilePosition) }
            )
            
            // Spacer(modifier = Modifier.width(4.dp)) Disabled for dots and commas
            
            Text(
                text = when (selectedMessage) {
                    "pizzaPre" -> ". We hope you brought pizza."
                    "everyoneWelcomePre" -> "!"
                    "slid" -> " just slid into the server!"
                    "showedUp" -> " just showed up!"
                    "hopped" -> " hopped into the server."
                    else -> " just slid into the server!"
                },
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface
            )
            
            Spacer(modifier = Modifier.width(8.dp))
            
            MessageTimestamp(
                timestamp = message.timestamp,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        } else {
            Box(modifier = Modifier.width(40.dp), contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = iconTint
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Text(
                text = text,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            
            Spacer(modifier = Modifier.width(8.dp))
            
            MessageTimestamp(
                timestamp = message.timestamp,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
    }
}
