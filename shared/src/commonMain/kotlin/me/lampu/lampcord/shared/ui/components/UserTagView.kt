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

@Composable
fun UserTagView(
    user: User,
    modifier: Modifier = Modifier,
    alpha: Float = 1f
) {
    val isBot = user.bot == true
    val isSystem = user.system == true
    
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
