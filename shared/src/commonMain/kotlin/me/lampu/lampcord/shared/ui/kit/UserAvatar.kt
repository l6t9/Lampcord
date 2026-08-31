package me.lampu.lampcord.shared.ui.kit

import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.AvatarDecorationData
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration

@Composable
fun UserAvatar(
    user: User?,
    size: Dp = 40.dp,
    modifier: Modifier = Modifier,
    decorationData: AvatarDecorationData? = null
) {
    val avatarUrl = user?.avatar?.let {
        val extension = if (it.startsWith("a_")) "gif" else "png"
        "https://cdn.discordapp.com/avatars/${user.id}/$it.$extension?size=${(size.value * 2).toInt()}"
    } ?: user?.let {
        val index = ((it.id.toLongOrNull() ?: 0L) shr 22) % 6
        "https://cdn.discordapp.com/embed/avatars/$index.png"
    }

    val fallbackUrl = user?.let {
        val index = ((it.id.toLongOrNull() ?: 0L) shr 22) % 6
        "https://cdn.discordapp.com/embed/avatars/$index.png"
    }
    AvatarWithDecoration(
        avatarUrl = avatarUrl,
        fallbackAvatarUrl = fallbackUrl,
        decorationData = decorationData ?: user?.avatar_decoration_data,
        size = size,
        modifier = modifier
    )
}
