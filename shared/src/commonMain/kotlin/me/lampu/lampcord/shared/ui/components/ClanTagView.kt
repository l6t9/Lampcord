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
import me.lampu.lampcord.shared.model.PrimaryGuild

@Composable
fun ClanTagView(
    primaryGuild: PrimaryGuild?,
    modifier: Modifier = Modifier,
    alpha: Float = 1f
) {
    if (primaryGuild?.clan_tag == null) return

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(4.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f * alpha))
            .padding(horizontal = 4.dp, vertical = 1.dp),
        contentAlignment = Alignment.Center
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (primaryGuild.clan_badge != null) {
                val badgeUrl = "https://cdn.discordapp.com/clan-badges/${primaryGuild.guild_id}/${primaryGuild.clan_badge}.png?size=16"
                AsyncImage(
                    model = badgeUrl,
                    contentDescription = null,
                    modifier = Modifier.size(12.dp)
                )
                Spacer(Modifier.width(2.dp))
            }
            Text(
                text = primaryGuild.clan_tag,
                style = MaterialTheme.typography.labelSmall.copy(
                    fontSize = 10.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.5.sp
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = alpha)
            )
        }
    }
}
