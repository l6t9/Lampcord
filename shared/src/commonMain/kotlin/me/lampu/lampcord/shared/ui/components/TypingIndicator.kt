package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.TypingStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun TypingIndicator(
    modifier: Modifier = Modifier,
    typingStore: TypingStore = koinInject(),
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val typingUsers by typingStore.typingUsers.collectAsState()
    val channelId = navigationStore.selectedThread?.id ?: navigationStore.selectedChannel?.id ?: return
    val guild = navigationStore.selectedGuild
    if (guild != null && (guild.verification_level ?: 0) > 0) return
    val typingMap = typingUsers[channelId] ?: return
    val userIds = typingMap.keys.toList()
    if (userIds.isEmpty()) return
    
    val names = userIds.take(4).map { id ->
         val member = navigationStore.selectedGuild?.let { userStore.getMember(it.id, id) }
         val userFromStore = userStore.getUser(id)
         val userFromChannel = navigationStore.selectedChannel?.recipients?.find { it.id == id }
         
         member?.nick 
         ?: userFromStore?.global_name 
         ?: userFromStore?.username 
         ?: userFromChannel?.global_name
         ?: userFromChannel?.username
         ?: "Someone"
    }
    
    val text = when (names.size) {
        1 -> "${names[0]} is typing\u2026"
        2 -> "${names[0]} and ${names[1]} are typing\u2026"
        3 -> "${names[0]}, ${names[1]}, and ${names[2]} are typing\u2026"
        else -> "Several people are typing\u2026"
    }

    Surface(
        modifier = modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Row(
            modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 2.dp, bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            TypingDots(modifier = Modifier.size(12.dp))
            Text(
                text = text,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontWeight = FontWeight.Medium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

@Composable
fun TypingDots(modifier: Modifier = Modifier) {
    // Keep the typing indicator visible without a continuously animated clock on Windows.
    if (getPlatformName() == "windows" || Settings.shared.reduceMotion) {
        Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
            repeat(3) {
                Box(
                    Modifier
                        .size(4.dp)
                        .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.65f), CircleShape)
                )
            }
        }
        return
    }

    val dotColor = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.65f)
    val cycle = 1200
    val stagger = cycle / 3

    val infiniteTransition = rememberInfiniteTransition(label = "typingDots")
    val scale1 by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes {
                durationMillis = cycle
                1f at 0
                1.45f at stagger
                1f at stagger * 2
                1f at cycle
            },
            repeatMode = RepeatMode.Restart
        ),
        label = "scale1"
    )
    val scale2 by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes {
                durationMillis = cycle
                1f at 0
                1f at stagger
                1.45f at stagger * 2
                1f at cycle
            },
            repeatMode = RepeatMode.Restart
        ),
        label = "scale2"
    )
    val scale3 by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes {
                durationMillis = cycle
                1f at 0
                1f at stagger * 2
                1.45f at stagger * 3 - stagger / 2
                1f at cycle
            },
            repeatMode = RepeatMode.Restart
        ),
        label = "scale3"
    )

    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(2.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            Modifier
                .size(4.dp)
                .graphicsLayer { scaleX = scale1; scaleY = scale1 }
                .background(dotColor, CircleShape)
        )
        Box(
            Modifier
                .size(4.dp)
                .graphicsLayer { scaleX = scale2; scaleY = scale2 }
                .background(dotColor, CircleShape)
        )
        Box(
            Modifier
                .size(4.dp)
                .graphicsLayer { scaleX = scale3; scaleY = scale3 }
                .background(dotColor, CircleShape)
        )
    }
}
