package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.User

import androidx.compose.material3.Icon
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun UserTagView(
    user: User,
    modifier: Modifier = Modifier,
    alpha: Float = 1f
) {
    val isBot = user.bot == true
    val isSystem = user.system == true
    val isVerifiedBot = ((user.public_flags ?: 0) or (user.flags ?: 0)) and 65536 != 0
    
    if (!isBot && !isSystem) return

    val tagText = if (isSystem) "SYSTEM" else "BOT"
    val backgroundColor = if (isSystem) Color(0xFFFFB000) else Color(0xFF5865F2)

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(3.dp))
            .background(backgroundColor.copy(alpha = alpha))
            .padding(horizontal = 4.dp, vertical = 1.dp),
        contentAlignment = Alignment.Center
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (isVerifiedBot && !isSystem) {
                Icon(
                    imageVector = Icons.Filled.Check,
                    contentDescription = null,
                    modifier = Modifier.size(8.dp),
                    tint = Color.White.copy(alpha = alpha)
                )
                Spacer(Modifier.width(2.dp))
            }
            Text(
                text = tagText,
                style = MaterialTheme.typography.labelSmall.copy(
                    fontSize = 9.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.5.sp
                ),
                color = Color.White.copy(alpha = alpha)
            )
        }
    }
}
