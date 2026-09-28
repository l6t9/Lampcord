package me.lampu.lampcord.shared.ui.kit

import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalDensity
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
    guildId: String? = null,
    memberAvatar: String? = null,
    decorationData: AvatarDecorationData? = null,
    isHovered: Boolean = false,
    forceAnimate: Boolean = false
) {
    val sizePx = with(LocalDensity.current) { size.roundToPx() }
    val avatarUrl = remember(user, guildId, memberAvatar, sizePx) {
        if (user == null) return@remember null
        if (guildId != null && memberAvatar != null) {
            me.lampu.lampcord.shared.api.CdnUrls.getMemberAvatarUrl(
                guildId,
                user.id,
                memberAvatar,
                user.avatar,
                sizePx
            )
        } else {
            me.lampu.lampcord.shared.api.CdnUrls.getUserAvatarUrl(user.id, user.avatar, sizePx)
        }
    }

    val fallbackUrl = user?.let {
        val index = ((it.id.toLongOrNull() ?: 0L) shr 22) % 6
        "https://cdn.discordapp.com/embed/avatars/$index.png"
    }
    AvatarWithDecoration(
        avatarUrl = avatarUrl,
        fallbackAvatarUrl = fallbackUrl,
        decorationData = decorationData ?: user?.avatar_decoration_data
            ?: user?.collectibles?.avatar_decoration,
        size = size,
        modifier = modifier,
        isHovered = isHovered,
        forceAnimate = forceAnimate
    )
}
