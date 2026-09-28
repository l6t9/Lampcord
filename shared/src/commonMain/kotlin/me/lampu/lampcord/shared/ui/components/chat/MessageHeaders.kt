package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RichTooltip
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.material3.TooltipBox
import androidx.compose.material3.TooltipDefaults
import androidx.compose.material3.rememberTooltipState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.DiscordMarkdownText
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.api.CdnUrls
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MessageTimestamp(
    timestamp: String,
    style: androidx.compose.ui.text.TextStyle,
    color: Color,
    modifier: Modifier = Modifier
) {
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
            color = color,
            modifier = modifier,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis
        )
    }
}

@Composable
fun InteractionHeader(
    interaction: MessageInteraction,
    guildId: String? = null,
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
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

        val avatarUrl = remember(interaction, guildId, navigationStore.selectedGuild) {
            val user = interaction.user ?: return@remember null
            val gId = guildId ?: navigationStore.selectedGuild?.id
            val member = if (gId != null) userStore.getMember(gId, user.id) else null
            
            if (gId != null && member?.avatar != null) {
                CdnUrls.getMemberAvatarUrl(gId, user.id, member.avatar, user.avatar, 48)
            } else {
                CdnUrls.getUserAvatarUrl(user.id, user.avatar, 48)
            }
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = null,
                modifier = Modifier.size(16.dp).clip(CircleShape),
                filterQuality = FilterQuality.Medium,
                allowAnimation = false
            )
        } else {
            Surface(modifier = Modifier.size(16.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
        }

        Spacer(modifier = Modifier.width(4.dp))

        Text(
            text = interaction.user?.global_name ?: interaction.user?.username ?: "Unknown User",
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
fun ReplyBar(
    referencedMessage: Message,
    settingsStore: SettingsStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    val lineColor = if (settingsStore.pureBlack) Color.DarkGray else MaterialTheme.colorScheme.outlineVariant
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
            .clickableCursor(
                interactionSource = remember { androidx.compose.foundation.interaction.MutableInteractionSource() },
                indication = null // Removed the highlight indication
            ) {
                messageStore.scrollToMessageId = referencedMessage.id
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

        val avatarUrl = remember(referencedMessage, navigationStore.selectedGuild) {
            val author = referencedMessage.author ?: return@remember null
            val guildId = referencedMessage.guild_id ?: navigationStore.selectedGuild?.id
            val memberAvatar = referencedMessage.member?.avatar
            
            if (guildId != null && memberAvatar != null) {
                CdnUrls.getMemberAvatarUrl(guildId, author.id, memberAvatar, author.avatar, 48)
            } else {
                CdnUrls.getUserAvatarUrl(author.id, author.avatar, 48)
            }
        }

        if (avatarUrl != null) {
            AsyncImage(
                model = avatarUrl,
                contentDescription = null,
                modifier = Modifier.size(16.dp).clip(CircleShape),
                filterQuality = FilterQuality.Medium,
                allowAnimation = false
            )
        } else {
            Surface(modifier = Modifier.size(16.dp), shape = CircleShape, color = MaterialTheme.colorScheme.primaryContainer) {}
        }

        Spacer(modifier = Modifier.width(4.dp))

        val roleColor = remember(referencedMessage, navigationStore.selectedGuild) {
            val guild = navigationStore.selectedGuild ?: return@remember Color.White
            val authorId = referencedMessage.author?.id ?: return@remember Color.White
            val member = referencedMessage.member ?: userStore.getMember(guild.id, authorId) ?: return@remember Color.White
            val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
            val colorRole = memberRoles.filter { it.color != 0 }.maxByOrNull { it.position }
            if (colorRole != null) Color(colorRole.color or 0xFF000000.toInt()) else Color.White
        }

        Text(
            text = referencedMessage.author?.global_name ?: referencedMessage.author?.username ?: "Unknown User",
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = if (isHovered) Color.White else roleColor.copy(alpha = 0.8f), // Whiter on hover
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f, fill = false)
        )

        referencedMessage.author?.let { author ->
            author.primary_guild?.let {
                Spacer(Modifier.width(4.dp))
                ClanTagView(it, alpha = if (isHovered) 1f else 0.7f)
            }
            UserTagView(author, modifier = Modifier.padding(start = 4.dp), alpha = if (isHovered) 1f else 0.7f)
        }

        Spacer(modifier = Modifier.width(4.dp))

        val replyContent = when {
            referencedMessage.content.isNotBlank() -> referencedMessage.content
            referencedMessage.attachments.isNotEmpty() -> "Click to see attachment"
            referencedMessage.embeds.isNotEmpty() -> "Click to see embed"
            else -> "Original message"
        }

        DiscordMarkdownText(
            content = replyContent,
            style = MaterialTheme.typography.labelSmall,
            color = if (isHovered) Color.White.copy(alpha = 0.6f) else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f),
            modifier = Modifier.weight(1f, fill = false),
            maxLines = 1
        )
    }
}

@Composable
fun ThreadStarterBar(
    thread: me.lampu.lampcord.shared.model.Channel,
    navigationStore: NavigationStore = koinInject()
) {
    Surface(
        onClick = { navigationStore.selectThread(thread, explicitlySelected = true) },
        modifier = Modifier
            .padding(top = 8.dp)
            .fillMaxWidth(0.8f),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.4f),
        shape = androidx.compose.foundation.shape.RoundedCornerShape(8.dp),
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            androidx.compose.material3.Icon(
                imageVector = me.lampu.lampcord.shared.ui.icons.Icons.Rounded.Topic,
                contentDescription = null,
                modifier = Modifier.size(18.dp),
                tint = MaterialTheme.colorScheme.primary
            )
            Spacer(Modifier.width(8.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = thread.name ?: "Thread",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                if (thread.message_count != null) {
                    Text(
                        text = "${thread.message_count} messages",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            androidx.compose.material3.Icon(
                imageVector = me.lampu.lampcord.shared.ui.icons.Icons.Rounded.Topic,
                contentDescription = null,
                modifier = Modifier.size(16.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
    }
}
