package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ChannelNavigator
import me.lampu.lampcord.shared.state.NotificationStore
import org.koin.compose.koinInject

@Composable
fun InAppNotificationHost(
    modifier: Modifier = Modifier,
    notificationStore: NotificationStore = koinInject(),
    channelNavigator: ChannelNavigator = koinInject()
) {
    val toasts by notificationStore.toasts.collectAsState()

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp)
            .padding(top = 8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        toasts.forEach { toast ->
            val data = toast.data
            AnimatedVisibility(
                visible = true,
                enter = if (Settings.shared.reduceMotion) EnterTransition.None else slideInVertically { -it } + fadeIn(),
                exit = if (Settings.shared.reduceMotion) ExitTransition.None else slideOutVertically { -it } + fadeOut()
            ) {
                val preview = when {
                    data.message.content.isNotBlank() -> data.message.content
                    data.message.attachments.isNotEmpty() -> "Sent an attachment"
                    data.message.sticker_items?.isNotEmpty() == true -> "Sent a sticker"
                    data.message.embeds.isNotEmpty() -> data.message.embeds.first().title ?: "Sent an embed"
                    else -> "New message"
                }

                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .shadow(16.dp, RoundedCornerShape(16.dp))
                        .clip(RoundedCornerShape(16.dp))
                        .background(MaterialTheme.colorScheme.surfaceContainerHigh)
                        .clickable {
                            notificationStore.dismiss(toast.id)
                            channelNavigator.navigateToChannel(data.message.channel_id, data.message.guild_id)
                        }
                        .padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    AsyncImage(
                        model = data.authorAvatarUrl,
                        contentDescription = null,
                        modifier = Modifier
                            .size(40.dp)
                            .clip(CircleShape),
                        showPlaceholder = false
                    )
                    Column(modifier = Modifier.weight(1f)) {
                        val title = buildString {
                            append(data.authorDisplayName)
                            data.channelLabel?.let { append("  ·  $it") }
                        }
                        Text(
                            text = title,
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurface,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                        Text(
                            text = if (Settings.shared.showMessagePreview) preview else "New message",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
        }
    }
}
