package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.MessageComponent
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.theme.DiscordGreen
import me.lampu.lampcord.shared.ui.theme.DiscordRed

@Composable
fun SelectMenuView(component: MessageComponent) {
    Surface(
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 425.dp).fillMaxWidth().height(40.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shape = RoundedCornerShape(4.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant),
        onClick = { /* TODO */ },
        enabled = component.disabled != true
    ) {
        Row(modifier = Modifier.padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text(text = component.placeholder ?: "Select an option...", style = MaterialTheme.typography.bodyMedium, color = if (component.disabled == true) MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f) else MaterialTheme.colorScheme.onSurfaceVariant)
            Icon(imageVector = Icons.Filled.ExpandMore, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
fun MessageComponentsRow(
    components: List<MessageComponent>
) {
    androidx.compose.foundation.layout.FlowRow(
        modifier = Modifier.padding(top = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        components.forEach { component ->
            when (component.type) {
                1 -> { component.components?.let { MessageComponentsRow(it) } }
                2 -> {
                    val isLink = component.style == 5
                    val buttonColor = when (component.style) {
                        1 -> ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary, contentColor = MaterialTheme.colorScheme.onPrimary)
                        2 -> ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.secondaryContainer, contentColor = MaterialTheme.colorScheme.onSecondaryContainer)
                        3 -> ButtonDefaults.buttonColors(containerColor = DiscordGreen, contentColor = Color.White)
                        4 -> ButtonDefaults.buttonColors(containerColor = DiscordRed, contentColor = Color.White)
                        else -> ButtonDefaults.buttonColors()
                    }
                    val uriHandler = LocalUriHandler.current
                    Button(
                        onClick = { if (isLink && component.url != null) uriHandler.openUri(component.url) },
                        colors = buttonColor,
                        shape = RoundedCornerShape(4.dp),
                        modifier = Modifier.height(32.dp),
                        contentPadding = PaddingValues(horizontal = 12.dp),
                        enabled = component.disabled != true
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            component.emoji?.let { emoji ->
                                val emojiUrl = emoji.id?.let { "https://cdn.discordapp.com/emojis/$it.webp?size=48&animated=${emoji.animated == true}" }
                                if (emojiUrl != null) { AsyncImage(model = emojiUrl, contentDescription = null, modifier = Modifier.size(16.dp)) }
                                else { Text(emoji.name ?: "", fontSize = 14.sp) }
                            }
                            component.label?.let { Text(it, style = MaterialTheme.typography.labelMedium) }
                            if (isLink) { Icon(Icons.AutoMirrored.Filled.OpenInNew, null, modifier = Modifier.size(14.dp)) }
                        }
                    }
                }
                3, 5, 6, 7, 8 -> { SelectMenuView(component) }
            }
        }
    }
}
