package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.graphics.vector.ImageVector
import me.lampu.lampcord.shared.model.ProfileBadge
import me.lampu.lampcord.shared.ui.icons.IconsBrand
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ExpressiveTooltip
import me.lampu.lampcord.shared.ui.components.tooltipText
import me.lampu.lampcord.shared.utils.showToast
import kotlin.math.abs
import kotlin.math.pow
import kotlin.math.roundToInt

data class ProfileTheme(
    val backgroundBrush: Brush,
    val outerBorderBrush: Brush,
    val bodyOverlayColor: Color,
    val cardColor: Color,
    val tagColor: Color,
    val contentColor: Color,
    val cutoutColor: Color,
    val pfpBorderBrush: Brush,
    val primaryAccent: Color,
    val buttonColor: Color,
    val buttonTextColor: Color,
    val isCustom: Boolean = false,
    val themeColors: List<Color> = emptyList()
)

@Composable
fun UserBadges(userId: String, badges: List<ProfileBadge>) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.padding(vertical = 4.dp)
    ) {
        badges.forEach { badge ->
            val iconUrl = "https://cdn.discordapp.com/badge-icons/${badge.icon}.png?size=64"
            ExpressiveTooltip(
                anchorPosition = TooltipAnchorPosition.Above,
                maxWidth = 150,
                content = tooltipText(badge.description),
                anchor = {
                    AsyncImage(
                        model = iconUrl,
                        contentDescription = badge.description,
                        modifier = Modifier
                            .size(22.dp)
                            .clickable { showToast(badge.description) }
                    )
                }
            )
        }
        me.lampu.lampcord.shared.ui.components.CustomBadgesView(userId, badgeSize = 22.dp, spacing = 4.dp)
    }
}

fun getConnectionIcon(type: String, name: String? = null): ImageVector {
    val normalizedType = type.lowercase()
    if (normalizedType == "website" || normalizedType == "domain") return Icons.Rounded.Language
    
    // Check name if type is unknown or generic
    if (name != null && name.contains(".") && !name.contains(" ")) return Icons.Rounded.Language

    return when (normalizedType) {
        "github" -> IconsBrand.Github
        "spotify" -> IconsBrand.Spotify
        "steam" -> IconsBrand.Steam
        "twitch" -> IconsBrand.Twitch
        "reddit" -> IconsBrand.Reddit
        "twitter" -> IconsBrand.Twitter
        "xbox" -> IconsBrand.Xbox
        "playstation" -> IconsBrand.PlayStation
        "youtube" -> IconsBrand.Youtube
        "instagram" -> IconsBrand.Instagram
        "telegram" -> IconsBrand.Telegram
        "battlenet" -> IconsBrand.BuyMeACoffee // Placeholder
        "facebook" -> Icons.Rounded.Public // Placeholder
        "mastodon" -> Icons.Rounded.Public // Placeholder
        "tiktok" -> Icons.Rounded.Public // Placeholder
        else -> Icons.Rounded.Link
    }
}

fun getConnectionUrl(type: String, name: String, id: String): String? {
    val normalizedType = type.lowercase()
    return when (normalizedType) {
        "github" -> "https://github.com/$name"
        "spotify" -> "https://open.spotify.com/user/$id"
        "steam" -> "https://steamcommunity.com/profiles/$id"
        "twitch" -> "https://twitch.tv/$name"
        "reddit" -> "https://reddit.com/u/$name"
        "twitter" -> "https://twitter.com/$name"
        "youtube" -> "https://youtube.com/channel/$id"
        "instagram" -> "https://instagram.com/$name"
        "telegram" -> "https://t.me/$name"
        "facebook" -> "https://facebook.com/$id"
        "tiktok" -> "https://tiktok.com/@$name"
        "mastodon" -> if (name.contains("@")) {
            val parts = name.split("@")
            if (parts.size >= 3) "https://${parts[2]}/@${parts[1]}" else null
        } else null
        "website", "domain" -> if (name.startsWith("http")) name else "https://$name"
        else -> {
            // Fallback: If it looks like a URL or a domain, try to link it
            if (name.startsWith("http")) {
                name
            } else if (name.contains(".") && !name.contains(" ")) {
                "https://$name"
            } else {
                null
            }
        }
    }
}

object ModernProfileColors {
    fun getLuminance(color: Int): Double {
        val r = (color shr 16) and 0xFF
        val g = (color shr 8) and 0xFF
        val b = color and 0xFF

        fun channel(c: Int): Double {
            val v = c / 255.0
            return if (v <= 0.03928) v / 12.92 else ((v + 0.055) / 1.055).pow(2.4)
        }
        return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
    }

    fun rgbToHsl(color: Int): Triple<Double, Double, Double> {
        val r = ((color shr 16) and 0xFF) / 255.0
        val g = ((color shr 8) and 0xFF) / 255.0
        val b = (color and 0xFF) / 255.0
        val max = maxOf(r, g, b)
        val min = minOf(r, g, b)
        var h = 0.0
        val l = (max + min) / 2.0
        val d = max - min
        val s = if (d == 0.0) 0.0 else d / (1.0 - abs(2.0 * l - 1.0))
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
        val c = (1 - abs(2 * l - 1)) * s
        val hh = h / 60.0
        val x = c * (1 - abs(hh % 2 - 1))
        val (r1, g1, b1) = when {
            hh < 1 -> Triple(c, x, 0.0)
            hh < 2 -> Triple(x, c, 0.0)
            hh < 3 -> Triple(0.0, c, x)
            hh < 4 -> Triple(0.0, x, c)
            hh < 5 -> Triple(x, 0.0, c)
            else -> Triple(c, 0.0, x)
        }
        val m = l - c / 2.0
        val rr = ((r1 + m) * 255.0).roundToInt().coerceIn(0, 255)
        val rg = ((g1 + m) * 255.0).roundToInt().coerceIn(0, 255)
        val rb = ((b1 + m) * 255.0).roundToInt().coerceIn(0, 255)
        return (0xFF shl 24) or (rr shl 16) or (rg shl 8) or rb
    }

    fun mix(a: Int, b: Int, t: Double): Int {
        val ar = (a shr 16) and 0xFF
        val ag = (a shr 8) and 0xFF
        val ab = a and 0xFF
        val br = (b shr 16) and 0xFF
        val bg = (b shr 8) and 0xFF
        val bb = b and 0xFF
        val rr = (ar * (1 - t) + br * t).roundToInt().coerceIn(0, 255)
        val rg = (ag * (1 - t) + bg * t).roundToInt().coerceIn(0, 255)
        val rb = (ab * (1 - t) + bb * t).roundToInt().coerceIn(0, 255)
        return 0xFF000000.toInt() or (rr shl 16) or (rg shl 8) or rb
    }
}
