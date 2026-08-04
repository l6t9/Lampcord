package com.example.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import com.example.lampcord.shared.model.UserProfile
import com.example.lampcord.shared.state.ChatState
import com.example.lampcord.shared.model.ProfileBadge
import com.example.lampcord.shared.ui.icons.Icons

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
fun UserProfileDialog(
    profile: UserProfile,
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    
    val themeColors = remember(profile) {
        val rawColors = guildMeta?.theme_colors ?: userMeta?.theme_colors
        val base = if (!rawColors.isNullOrEmpty()) rawColors else {
            val accent = guildMeta?.accent_color ?: userMeta?.accent_color ?: user.accent_color
            if (accent != null) listOf(accent) else null
        }
        
        base?.let { colors ->
            if (colors.size >= 2) colors else {
                val c = colors[0]
                val (h, s, l) = ModernProfileColors.rgbToHsl(c)
                val c2 = ModernProfileColors.hslToRgb((h + 20) % 360, (s * 1.2).coerceIn(0.0, 1.0), (l * 0.8).coerceIn(0.0, 1.0))
                listOf(c, c2)
            }
        }
    }

    val theme = remember(themeColors) {
        if (themeColors != null && themeColors.size >= 2) {
            val primary = themeColors[0]
            val accent = themeColors[1]
            
            val (h1, s1, l1) = ModernProfileColors.rgbToHsl(primary)
            val isLightMode = l1 > 0.6

            // 1. Background Gradient (Darkened strictly from HSL for saturation and depth)
            val bg1 = Color(ModernProfileColors.hslToRgb(h1, s1, (l1 * 0.65).coerceIn(0.0, 1.0)))
            val (h2, s2, l2) = ModernProfileColors.rgbToHsl(accent)
            val bg2 = Color(ModernProfileColors.hslToRgb(h2, s2, (l2 * 0.65).coerceIn(0.0, 1.0)))
            
            // 2. Outer Border (Lighter and more saturated versions for that "Accent" look)
            val b1 = Color(ModernProfileColors.hslToRgb(h1, (s1 * 1.15).coerceIn(0.0, 1.0), (l1 * 1.3).coerceIn(0.0, 1.0)))
            val b2 = Color(ModernProfileColors.hslToRgb(h2, (s2 * 1.15).coerceIn(0.0, 1.0), (l2 * 1.3).coerceIn(0.0, 1.0)))
            
            ProfileTheme(
                backgroundBrush = Brush.verticalGradient(0.0f to bg1, 0.35f to bg1, 1.0f to bg2),
                outerBorderBrush = Brush.verticalGradient(listOf(b1, b2)),
                bodyOverlayColor = if (isLightMode) Color.White.copy(alpha = 0.3f) else Color.Black.copy(alpha = 0.45f),
                contentColor = if (isLightMode) Color.Black else Color.White,
                cutoutColor = bg1,
                pfpBorderBrush = Brush.verticalGradient(listOf(Color(primary or 0xFF000000.toInt()), Color(accent or 0xFF000000.toInt()))),
                primaryAccent = Color(primary or 0xFF000000.toInt()),
                buttonColor = Color(primary or 0xFF000000.toInt())
            )
        } else {
            ProfileTheme(
                backgroundBrush = Brush.verticalGradient(listOf(Color(0xFF18191c), Color(0xFF111214))),
                outerBorderBrush = Brush.verticalGradient(listOf(Color.White.copy(alpha = 0.2f), Color.White.copy(alpha = 0.1f))),
                bodyOverlayColor = Color.Black.copy(alpha = 0.45f),
                contentColor = Color.White,
                cutoutColor = Color(0xFF18191c),
                pfpBorderBrush = Brush.verticalGradient(listOf(Color(0xFF5865F2), Color(0xFF5865F2))),
                primaryAccent = Color(0xFF5865F2),
                buttonColor = Color(0xFF5865F2)
            )
        }
    }

    val popupPosition = remember(chatState.profilePosition) {
        chatState.profilePosition ?: androidx.compose.ui.geometry.Offset.Zero
    }

    Popup(
        offset = IntOffset(
            x = if (popupPosition.x < 500) (popupPosition.x + 60).toInt() else (popupPosition.x - 320).toInt(),
            y = (popupPosition.y.toInt() - 100).coerceAtLeast(10)
        ),
        onDismissRequest = onDismiss,
        properties = PopupProperties(focusable = true, dismissOnClickOutside = true)
    ) {
        Box(
            modifier = Modifier
                .width(300.dp)
                .wrapContentHeight()
                .background(theme.outerBorderBrush, RoundedCornerShape(10.dp))
                .padding(3.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(7.dp))
                    .background(theme.backgroundBrush)
            ) {
                // Scrollable Body
                Column(modifier = Modifier.weight(1f, fill = false)) {
                    // Header
                    Box(modifier = Modifier.fillMaxWidth().height(105.dp)) {
                        val bannerUrl = guildMeta?.banner ?: userMeta?.banner ?: user.banner
                        if (bannerUrl != null) {
                            AsyncImage(
                                model = "https://cdn.discordapp.com/banners/${user.id}/$bannerUrl.png?size=600",
                                contentDescription = null,
                                contentScale = ContentScale.Crop,
                                modifier = Modifier.fillMaxSize(),
                                filterQuality = FilterQuality.Medium
                            )
                        } else {
                            MeshGradientBackground(
                                colors = listOf(
                                    theme.primaryAccent,
                                    theme.primaryAccent.copy(alpha = 0.8f),
                                    theme.buttonColor.copy(alpha = 0.6f),
                                    theme.cutoutColor
                                )
                            )
                        }
                    }

                    Column(modifier = Modifier.padding(start = 10.dp, end = 16.dp)) {
                        // Avatar
                        val avatarUrl = profile.guild_member?.avatar?.let {
                            "https://cdn.discordapp.com/guilds/${chatState.selectedGuild?.id}/users/${user.id}/avatars/$it.png?size=160"
                        } ?: user.avatar?.let {
                            "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160"
                        }

                        Row(verticalAlignment = Alignment.Bottom) {
                            Box(
                                modifier = Modifier
                                    .offset(y = (-45).dp)
                                    .size(94.dp)
                                    .background(theme.cutoutColor, CircleShape)
                                    .padding(6.dp)
                            ) {
                                Surface(
                                    modifier = Modifier.fillMaxSize(),
                                    shape = CircleShape,
                                    color = Color.DarkGray
                                ) {
                                    if (avatarUrl != null) {
                                        AsyncImage(
                                            model = avatarUrl,
                                            contentDescription = null,
                                            modifier = Modifier.fillMaxSize(),
                                            filterQuality = FilterQuality.Medium
                                        )
                                    } else {
                                        Box(contentAlignment = Alignment.Center) {
                                            Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.headlineLarge)
                                        }
                                    }
                                }
                                val status = chatState.getUserStatus(user.id)
                                StatusIndicator(
                                    status = status,
                                    size = 24.dp,
                                    modifier = Modifier
                                        .align(Alignment.BottomEnd),
                                    borderColor = theme.cutoutColor,
                                    borderWidth = 4.dp,
                                    backgroundColor = theme.cutoutColor
                                )
                            }

                            val presence = profile.guild_member?.presence ?: chatState.presences[user.id]
                            val customStatus = presence?.activities?.find { it.type == 4 }

                            if (customStatus != null) {
                                UserActivity(
                                    activity = customStatus,
                                    compact = true,
                                    modifier = Modifier
                                        .offset(y = (-35).dp)
                                        .padding(start = 12.dp, bottom = 8.dp)
                                )
                            }
                        }

                        Column(modifier = Modifier.offset(y = (-35).dp)) {
                            Text(
                                text = profile.guild_member?.nick ?: user.global_name ?: user.username,
                                style = MaterialTheme.typography.titleLarge,
                                fontWeight = FontWeight.Bold,
                                color = Color.White
                            )
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Text(user.username, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor.copy(alpha = 0.9f))
                                val pronouns = guildMeta?.pronouns ?: userMeta?.pronouns ?: user.pronouns
                                if (!pronouns.isNullOrBlank()) {
                                    Text(" • $pronouns", style = MaterialTheme.typography.bodyMedium, color = theme.contentColor.copy(alpha = 0.7f), modifier = Modifier.padding(start = 4.dp))
                                }
                            }
                            Spacer(Modifier.height(8.dp))
                            UserBadges(badges = profile.badges + profile.guild_badges, flags = user.public_flags ?: 0)
                            HorizontalDivider(modifier = Modifier.padding(vertical = 12.dp), color = Color.White.copy(alpha = 0.08f))
                            val bio = (guildMeta?.bio ?: userMeta?.bio ?: user.bio)
                            if (!bio.isNullOrBlank()) {
                                DiscordMarkdownText(content = bio, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor, chatState = chatState)
                            }

                            val presence = profile.guild_member?.presence ?: chatState.presences[user.id]
                            val activities = presence?.activities ?: emptyList()
                            val otherActivities = activities.filter { it.type != 4 }

                            otherActivities.forEach { activity ->
                                Spacer(Modifier.height(16.dp))
                                UserActivity(activity, compact = false)
                            }

                            if (profile.guild_member?.roles?.isNotEmpty() == true) {
                                Spacer(Modifier.height(16.dp))
                                androidx.compose.foundation.layout.FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                    val guild = chatState.selectedGuild
                                    profile.guild_member.roles.mapNotNull { id -> guild?.roles?.find { it.id == id } }.sortedByDescending { it.position }.forEach { role ->
                                        RoleTag(role.name, if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else theme.contentColor)
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer (Sticky) - Only show for current user
                if (user.id == chatState.currentUser?.id) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(start = 16.dp, end = 16.dp, bottom = 12.dp)
                    ) {
                        Button(
                            onClick = { /* TODO */ },
                            modifier = Modifier.fillMaxWidth().height(32.dp),
                            colors = ButtonDefaults.buttonColors(containerColor = theme.buttonColor),
                            shape = RoundedCornerShape(4.dp),
                            contentPadding = PaddingValues(horizontal = 12.dp)
                        ) {
                            Icon(Icons.Filled.Edit, null, modifier = Modifier.size(16.dp), tint = Color.White)
                            Spacer(Modifier.width(8.dp))
                            Text("Edit Profile", style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Medium, color = Color.White)
                        }
                    }
                } else {
                    Spacer(Modifier.height(16.dp))
                }
            }
        }
    }
}

@Composable
fun RoleTag(name: String, color: Color) {
    Surface(
        color = Color.Black.copy(alpha = 0.2f),
        shape = RoundedCornerShape(4.dp),
        border = androidx.compose.foundation.BorderStroke(0.5.dp, color.copy(alpha = 0.3f))
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(modifier = Modifier.size(12.dp).background(color, CircleShape))
            Spacer(modifier = Modifier.width(6.dp))
            Text(name, style = MaterialTheme.typography.labelSmall, color = Color.White)
        }
    }
}

@Composable
fun UserBadges(badges: List<ProfileBadge>, flags: Int, modifier: Modifier = Modifier) {
    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        if (badges.isNotEmpty()) {
            badges.forEach { badge ->
                AsyncImage(
                    model = "https://cdn.discordapp.com/badge-icons/${badge.icon}.png?size=48",
                    contentDescription = badge.description,
                    modifier = Modifier.size(20.dp),
                    filterQuality = FilterQuality.Medium
                )
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
