package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Role
import me.lampu.lampcord.shared.model.toTwemojiUrl

@Composable
fun RoleIcon(
    role: Role?,
    modifier: Modifier = Modifier,
    size: Dp = 16.dp
) {
    if (role == null) return

    if (role.icon != null) {
        AsyncImage(
            model = "https://cdn.discordapp.com/role-icons/${role.id}/${role.icon}.png?size=${(size.value * 2).toInt()}",
            contentDescription = null,
            modifier = modifier.size(size)
        )
    } else if (role.unicode_emoji != null) {
        AsyncImage(
            model = role.unicode_emoji.toTwemojiUrl(),
            contentDescription = null,
            modifier = modifier.size(size)
        )
    }
}
