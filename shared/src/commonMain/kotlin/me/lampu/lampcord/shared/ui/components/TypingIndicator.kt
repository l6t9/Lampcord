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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.TypingStore
import me.lampu.lampcord.shared.state.UserStore
import org.koin.compose.koinInject

@Composable
fun TypingIndicator(
    modifier: Modifier = Modifier,
    typingStore: TypingStore = koinInject(),
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val typingUsers by typingStore.typingUsers.collectAsState()
    val channelId = navigationStore.selectedChannel?.id ?: return
    val typingMap = typingUsers[channelId] ?: return
    val userIds = typingMap.keys.toList()
    if (userIds.isEmpty()) return
    
    val names = userIds.map { id ->
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
        1 -> "${names[0]} is typing..."
        2 -> "${names[0]} and ${names[1]} are typing..."
        3 -> "${names[0]}, ${names[1]} and ${names[2]} are typing..."
        else -> "Several people are typing..."
    }

    Surface(
        modifier = modifier.fillMaxWidth().height(24.dp),
        color = Color.Transparent
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            TypingDots()
            Spacer(Modifier.width(8.dp))
            Text(
                text = text,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@Composable
fun TypingDots(modifier: Modifier = Modifier) {
    val infiniteTransition = rememberInfiniteTransition(label = "typingDots")
    val alpha1 by infiniteTransition.animateFloat(
        initialValue = 0.2f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes { durationMillis = 600; 0.2f at 0; 1f at 300; 0.2f at 600 },
            repeatMode = RepeatMode.Restart
        ),
        label = "alpha1"
    )
    val alpha2 by infiniteTransition.animateFloat(
        initialValue = 0.2f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes { durationMillis = 600; 0.2f at 150; 1f at 450; 0.2f at 600 },
            repeatMode = RepeatMode.Restart
        ),
        label = "alpha2"
    )
    val alpha3 by infiniteTransition.animateFloat(
        initialValue = 0.2f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = keyframes { durationMillis = 600; 0.2f at 300; 1f at 600; 0.2f at 600 },
            repeatMode = RepeatMode.Restart
        ),
        label = "alpha3"
    )

    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        Box(Modifier.size(4.dp).background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha1), CircleShape))
        Box(Modifier.size(4.dp).background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha2), CircleShape))
        Box(Modifier.size(4.dp).background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha3), CircleShape))
    }
}
