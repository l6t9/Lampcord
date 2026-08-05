package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.ConnectedAccount
import me.lampu.lampcord.shared.model.ProfileBadge
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons

data class ProfileTheme(
    val backgroundBrush: Brush,
    val outerBorderBrush: Brush,
    val bodyOverlayColor: Color,
    val contentColor: Color,
    val cutoutColor: Color,
    val pfpBorderBrush: Brush,
    val primaryAccent: Color,
    val buttonColor: Color
)

@Composable
fun UserBadges(badges: List<ProfileBadge>, flags: Int) {
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        if (badges.isNotEmpty()) {
            badges.forEach { badge ->
                val iconUrl = "https://cdn.discordapp.com/badge-icons/${badge.icon}.png"
                AsyncImage(model = iconUrl, contentDescription = badge.description, modifier = Modifier.size(20.dp))
            }
        } else {
            if (flags and (1 shl 0) != 0) Badge(Color(0xFF5865F2)) // Staff
            if (flags and (1 shl 9) != 0) Badge(Color(0xFFFFCC00)) // Early Supporter
            if (flags and (1 shl 17) != 0) Badge(Color(0xFF40C4FF)) // Developer
            if (flags and (1 shl 22) != 0) Badge(Color(0xFF23A559)) // Active Developer
        }
    }
}

@Composable
fun Badge(color: Color) {
    Box(modifier = Modifier.size(18.dp).background(color.copy(alpha = 0.2f), CircleShape).padding(4.dp)) {
        Box(modifier = Modifier.fillMaxSize().background(color, CircleShape))
    }
}

@Composable
fun RoleTag(name: String, color: Color) {
    Surface(
        shape = RoundedCornerShape(4.dp),
        color = Color.Transparent,
        border = androidx.compose.foundation.BorderStroke(1.dp, color.copy(alpha = 0.5f)),
        modifier = Modifier.padding(vertical = 2.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(12.dp)
                    .background(color, CircleShape)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = name,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurface,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@Composable
fun UserConnectionItem(
    connection: ConnectedAccount,
    contentColor: Color,
    isFirst: Boolean,
    isLast: Boolean
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = Color.Black.copy(alpha = 0.1f),
        shape = when {
            isFirst && isLast -> RoundedCornerShape(8.dp)
            isFirst -> RoundedCornerShape(topStart = 8.dp, topEnd = 8.dp)
            isLast -> RoundedCornerShape(bottomStart = 8.dp, bottomEnd = 8.dp)
            else -> androidx.compose.ui.graphics.RectangleShape
        }
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val icon = when (connection.type) {
                "github" -> Icons.Brand.Github
                "steam" -> Icons.Brand.Steam
                "twitch" -> Icons.Brand.Twitch
                "youtube" -> Icons.Brand.Youtube
                else -> Icons.Filled.Link
            }
            Icon(icon, null, modifier = Modifier.size(24.dp), tint = contentColor)
            Spacer(Modifier.width(12.dp))
            Column {
                Text(connection.name, style = MaterialTheme.typography.bodyMedium, color = contentColor)
                Text(connection.type.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }, style = MaterialTheme.typography.labelSmall, color = contentColor.copy(alpha = 0.6f))
            }
            Spacer(Modifier.weight(1f))
            if (connection.verified) {
                Icon(Icons.Filled.Check, null, modifier = Modifier.size(16.dp), tint = Color(0xFF5865F2))
            }
        }
    }
}

object ModernProfileColors {
    fun rgbToHsl(color: Int): Triple<Double, Double, Double> {
        val r = ((color shr 16) and 0xFF) / 255.0
        val g = ((color shr 8) and 0xFF) / 255.0
        val b = (color and 0xFF) / 255.0
        val max = maxOf(r, g, b)
        val min = minOf(r, g, b)
        var h = 0.0
        val l = (max + min) / 2.0
        val d = max - min
        val s = if (d == 0.0) 0.0 else d / (1.0 - kotlin.math.abs(2.0 * l - 1.0))
        if (d != 0.0) {
            h = when (max) {
                r -> ((g - b) / d) % 6.0
                g -> ((b - r) / d) + 2.0
                else -> ((r - g) / d) + 4.0
            }
            h *= 60.0
            if (h < 0) h += 360.0
        }
        return Triple(h, s, l)
    }

    fun hslToRgb(h: Double, s: Double, l: Double): Int {
        val c = (1 - kotlin.math.abs(2 * l - 1)) * s
        val hh = h / 60.0
        val x = c * (1 - kotlin.math.abs(hh % 2 - 1))
        val (r1, g1, b1) = when {
            hh < 1 -> Triple(c, x, 0.0)
            hh < 2 -> Triple(x, c, 0.0)
            hh < 3 -> Triple(0.0, c, x)
            hh < 4 -> Triple(0.0, x, c)
            hh < 5 -> Triple(x, 0.0, c)
            else -> Triple(c, 0.0, x)
        }
        val m = l - c / 2.0
        val rr = ((r1 + m) * 255.0).toInt().coerceIn(0, 255)
        val rg = ((g1 + m) * 255.0).toInt().coerceIn(0, 255)
        val rb = ((b1 + m) * 255.0).toInt().coerceIn(0, 255)
        return (0xFF shl 24) or (rr shl 16) or (rg shl 8) or rb
    }

    fun mix(a: Int, b: Int, t: Double): Int {
        val ar = (a shr 16) and 0xFF
        val ag = (a shr 8) and 0xFF
        val ab = a and 0xFF
        val br = (b shr 16) and 0xFF
        val bg = (b shr 8) and 0xFF
        val bb = b and 0xFF
        val rr = kotlin.math.round(ar * (1 - t) + br * t).toInt().coerceIn(0, 255)
        val rg = kotlin.math.round(ag * (1 - t) + bg * t).toInt().coerceIn(0, 255)
        val rb = kotlin.math.round(ab * (1 - t) + bb * t).toInt().coerceIn(0, 255)
        return 0xFF000000.toInt() or (rr shl 16) or (rg shl 8) or rb
    }
}
