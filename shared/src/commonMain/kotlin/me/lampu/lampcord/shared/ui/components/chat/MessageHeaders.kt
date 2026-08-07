package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageInteraction
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.utils.DateTimeUtils

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MessageTimestamp(timestamp: String, style: androidx.compose.ui.text.TextStyle, color: Color) {
    val fullDate = remember(timestamp) { DateTimeUtils.formatFullDate(timestamp) }
    val displayDate = remember(timestamp) { DateTimeUtils.formatTimestamp(timestamp) }
    
    TooltipBox(
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(
            positioning = TooltipAnchorPosition.Above
        ),
        tooltip = {
            RichTooltip(
                caretShape = TooltipDefaults.caretShape()
            ) {
                val parts = fullDate.split(" at ")
                if (parts.size == 2) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(parts[0])
                        Text(parts[1])
                    }
                } else {
                    Text(fullDate)
            }
        }
        },
        state = rememberTooltipState()
    ) {
        Text(
            text = displayDate,
            style = style,
            color = color
        )
    }
}

@Composable
fun InteractionHeader(interaction: MessageInteraction) {
    val lineColor = MaterialTheme.colorScheme.outlineVariant
    Row(
        modifier = Modifier
            .padding(start = 8.dp, bottom = 4.dp)
            .height(24.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        androidx.compose.foundation.Canvas(
            modifier = Modifier
                .width(36.dp) 
                .fillMaxHeight()
        ) {
            val cornerRadius = 8.dp.toPx()
            val gutterX = 20.dp.toPx()
            val targetY = size.height / 2f
            
            val path = androidx.compose.ui.graphics.Path().apply {
                moveTo(gutterX, size.height + 8.dp.toPx()) 
                lineTo(gutterX, targetY + cornerRadius)
                quadraticTo(
                    gutterX, targetY,
                    gutterX + cornerRadius, targetY
                )
                lineTo(size.width, targetY)
            }

            drawPath(
                path = path,
                color = lineColor,
                style = androidx.compose.ui.graphics.drawscope.Stroke(
                    width = 1.5.dp.toPx(),
                    cap = androidx.compose.ui.graphics.StrokeCap.Round
                )
            )
        }

        Spacer(modifier = Modifier.width(4.dp))

        val avatarUrl = interaction.user.avatar?.let {
            "https://cdn.discordapp.com/avatars/${interaction.user.id}/$it.png?size=48"
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = null,
                modifier = Modifier.size(16.dp).clip(CircleShape),
                filterQuality = FilterQuality.Medium
            )
        } else {
            Surface(modifier = Modifier.size(16.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
        }

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = interaction.user.global_name ?: interaction.user.username,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f)
        )

        Spacer(modifier = Modifier.width(4.dp))
        
        Text(
            text = "used",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f)
        )

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = "/${interaction.name}",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.primary,
            fontWeight = FontWeight.Bold
        )
    }
}

@Composable
fun ReplyBar(referencedMessage: Message, chatState: ChatState) {
    val lineColor = if (chatState.settingsStore.pureBlack) Color.DarkGray else MaterialTheme.colorScheme.outlineVariant
    var isHovered by remember { mutableStateOf(false) }

    Row(
        modifier = Modifier
            .padding(start = 0.dp, bottom = 4.dp)
            .height(24.dp)
            .pointerInput(Unit) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent()
                        when (event.type) {
                            PointerEventType.Enter -> isHovered = true
                            PointerEventType.Exit -> isHovered = false
                        }
                    }
                }
            }
            .clickable(
                interactionSource = remember { androidx.compose.foundation.interaction.MutableInteractionSource() },
                indication = null // Removed the highlight indication
            ) {
                chatState.scrollToMessageId = referencedMessage.id
            },
        verticalAlignment = Alignment.CenterVertically
    ) {
        androidx.compose.foundation.Canvas(
            modifier = Modifier
                .width(36.dp) 
                .fillMaxHeight()
        ) {
            val cornerRadius = 8.dp.toPx()
            val gutterX = 18.dp.toPx() // Moved gutter slightly right (was 16dp, original 20dp)
            val targetY = size.height / 2f
            
            val path = androidx.compose.ui.graphics.Path().apply {
                moveTo(gutterX, size.height + 8.dp.toPx()) 
                lineTo(gutterX, targetY + cornerRadius)
                quadraticTo(
                    gutterX, targetY,
                    gutterX + cornerRadius, targetY
                )
                lineTo(size.width, targetY)
            }

            drawPath(
                path = path,
                color = if (isHovered) Color.White else lineColor, // Whiter indicator on hover
                style = androidx.compose.ui.graphics.drawscope.Stroke(
                    width = 1.5.dp.toPx(),
                    cap = androidx.compose.ui.graphics.StrokeCap.Round
                )
            )
        }

        Spacer(modifier = Modifier.width(4.dp))

        val avatarUrl = referencedMessage.member?.avatar?.let {
            "https://cdn.discordapp.com/guilds/${referencedMessage.guild_id ?: chatState.selectedGuild?.id}/users/${referencedMessage.author.id}/avatars/$it.png?size=48"
        } ?: referencedMessage.author.avatar?.let {
            "https://cdn.discordapp.com/avatars/${referencedMessage.author.id}/$it.png?size=48"
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = null,
                modifier = Modifier.size(16.dp).clip(CircleShape),
                filterQuality = FilterQuality.Medium
            )
        } else {
            Surface(modifier = Modifier.size(16.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
        }

        Spacer(modifier = Modifier.width(4.dp))

        val roleColor by remember(referencedMessage, chatState.selectedGuild) {
            derivedStateOf {
                val guild = chatState.selectedGuild ?: return@derivedStateOf Color.White
                val member = referencedMessage.member ?: chatState.getMember(guild.id, referencedMessage.author.id) ?: return@derivedStateOf Color.White
                val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
                val highestRole = memberRoles.maxByOrNull { it.position }
                if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.White
            }
        }

        Text(
            text = referencedMessage.author.global_name ?: referencedMessage.author.username,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = if (isHovered) Color.White else roleColor.copy(alpha = 0.8f) // Whiter on hover
        )

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = when {
                referencedMessage.content.isNotBlank() -> referencedMessage.content
                referencedMessage.attachments.isNotEmpty() -> "Click to see attachment"
                referencedMessage.embeds.isNotEmpty() -> "Click to see embed"
                else -> "Original message"
            },
            style = MaterialTheme.typography.labelSmall,
            color = if (isHovered) Color.White.copy(alpha = 0.6f) else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f), // Whiter on hover
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }
}
