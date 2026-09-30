package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.FinderResult
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.UserAvatar
import org.koin.compose.koinInject

/**
 * One search or picker row.
 *
 * A DM is labelled by the person on the other end rather than the literal "Direct Message" the
 * client used to fall back to, and a user result carries its status so offline members are still
 * distinguishable from the online ones.
 */
@Composable
fun FinderResultRow(
    result: FinderResult,
    onClick: () -> Unit,
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject()
) {
    val user = when (result) {
        is FinderResult.UserResult -> result.user
        is FinderResult.DirectMessage -> result.recipient
        else -> null
    }
    val currentUser by userStore.currentUser.collectAsState()
    val presences by presenceStore.presences.collectAsState()

    val status = user?.let {
        presenceStore.getUserStatus(it.id, presences[it.id], currentUser?.id, "online")
    }

    Surface(
        modifier = Modifier.fillMaxWidth(),
        onClick = onClick,
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (user != null) {
                UserAvatar(user = user, size = 40.dp)
                if (status != null) {
                    Box(modifier = Modifier.offset(x = 0.dp, y = 0.dp)) {
                        StatusIndicator(
                            status = status,
                            size = 14.dp,
                            borderColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
                        )
                    }
                }
            } else {
                Icon(
                    imageVector = when (result) {
                        is FinderResult.Guild -> Icons.Filled.Dns
                        is FinderResult.Channel -> when (result.channel.type) {
                            2, 13 -> Icons.Filled.VolumeUp
                            5 -> Icons.Filled.Campaign
                            else -> Icons.Filled.Tag
                        }

                        else -> Icons.Filled.Tag
                    },
                    contentDescription = null,
                    modifier = Modifier.size(24.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            Spacer(Modifier.width(16.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = result.title(),
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                result.subtitle()?.let { subtitle ->
                    Text(
                        text = subtitle,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }
    }
}

/** The channel id this result can forward to, if it names a destination at all. */
fun FinderResult.targetChannelId(): String? = when (this) {
    is FinderResult.Channel -> channel.id
    is FinderResult.DirectMessage -> channel.id
    is FinderResult.UserResult -> dmChannelId
    is FinderResult.Guild -> null
}

private fun FinderResult.title(): String = when (this) {
    is FinderResult.Guild -> guild.name ?: "Unnamed Server"
    is FinderResult.Channel -> channel.name ?: "unnamed-channel"
    is FinderResult.UserResult -> nickname ?: user.global_name ?: user.username ?: "Unknown user"
    is FinderResult.DirectMessage -> recipient?.let { it.global_name ?: it.username }
        ?: channel.name?.takeIf { it.isNotBlank() }
        ?: "Direct Message"
}

private fun FinderResult.subtitle(): String? = when (this) {
    is FinderResult.Guild -> "Server"
    is FinderResult.Channel -> guild?.name ?: "Channel"
    is FinderResult.UserResult -> user.username
    is FinderResult.DirectMessage -> "Direct Message"
}