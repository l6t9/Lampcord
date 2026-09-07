package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.*
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.PrimaryGuild
import me.lampu.lampcord.shared.ui.components.guilds.GuildProfileSheet

@Composable
fun ClanTagView(
    primaryGuild: PrimaryGuild?,
    modifier: Modifier = Modifier,
    alpha: Float = 1f
) {
    if (primaryGuild == null) return
    if (primaryGuild.identity_enabled == false) return
    
    val tag = primaryGuild.tag ?: return
    val badge = primaryGuild.badge
    val guildId = primaryGuild.identity_guild_id ?: primaryGuild.guild_id ?: return

    var showGuildProfile by remember { mutableStateOf(false) }

    Surface(
        modifier = modifier
            .clickable { showGuildProfile = true },
        color = MaterialTheme.colorScheme.surfaceContainerLow.copy(alpha = alpha),
        shape = RoundedCornerShape(4.dp),
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.1f * alpha))
    ) {
        val density = LocalDensity.current
        val maxTagWidthPx = remember(density) { with(density) { 50.dp.toPx() } }

        Row(
            modifier = Modifier
                .padding(horizontal = 4.dp, vertical = 1.dp)
                .widthIn(max = 50.dp)
                .graphicsLayer(compositingStrategy = CompositingStrategy.Offscreen)
                .drawWithContent {
                    drawContent()
                    if (size.width >= maxTagWidthPx - 1f) {
                        drawRect(
                            brush = Brush.horizontalGradient(
                                0.8f to Color.Black,
                                1f to Color.Transparent
                            ),
                            blendMode = BlendMode.DstIn
                        )
                    }
                },
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (badge != null) {
                val badgeUrl = "https://cdn.discordapp.com/guild-tag-badges/$guildId/$badge.png?size=32"
                AsyncImage(
                    model = badgeUrl,
                    contentDescription = null,
                    modifier = Modifier.size(14.dp)
                )
                Spacer(Modifier.width(2.dp))
            }
            Text(
                text = tag,
                style = MaterialTheme.typography.labelSmall.copy(
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.1.sp
                ),
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = alpha),
                maxLines = 1,
                overflow = TextOverflow.Clip,
                softWrap = false
            )
        }
    }

    if (showGuildProfile) {
        GuildProfileSheet(
            guildId = guildId,
            onDismiss = { showGuildProfile = false }
        )
    }
}
