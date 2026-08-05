package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.ProfileBadge
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.*

@Composable
fun UserProfileDialog(
    profile: UserProfile?,
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    val popupPosition = chatState.profilePosition
    val isExpanded = chatState.isProfileExpanded

    Popup(
        alignment = if (popupPosition == null || isExpanded) Alignment.Center else Alignment.TopStart,
        offset = if (popupPosition == null || isExpanded) IntOffset.Zero else IntOffset(
            x = if (popupPosition.x < 500) (popupPosition.x + 60).toInt() else (popupPosition.x - 320).toInt(),
            y = (popupPosition.y.toInt() - 100).coerceAtLeast(10)
        ),
        onDismissRequest = onDismiss,
        properties = PopupProperties(focusable = true, dismissOnClickOutside = true)
    ) {
        AnimatedVisibility(
            visible = profile != null,
            enter = fadeIn(tween(200, easing = LinearOutSlowInEasing)) + scaleIn(tween(200, easing = FastOutSlowInEasing), initialScale = 0.9f),
            exit = fadeOut(tween(150)) + scaleOut(tween(150), targetScale = 0.9f)
        ) {
            if (profile != null) {
                ProfileCard(
                    profile = profile,
                    chatState = chatState,
                    isExpanded = isExpanded,
                    onExpand = { chatState.isProfileExpanded = true },
                    modifier = Modifier.width(if (isExpanded) 600.dp else 300.dp).wrapContentHeight()
                )
            }
        }
    }
}

@Composable
fun ProfileCard(
    profile: UserProfile, 
    chatState: ChatState, 
    modifier: Modifier = Modifier,
    isExpanded: Boolean = false,
    onExpand: (() -> Unit)? = null,
    showBorder: Boolean = true
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

    val colorScheme = MaterialTheme.colorScheme
    val theme = remember(themeColors, colorScheme) {
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
            val primary = colorScheme.primary
            val surface = colorScheme.surfaceContainer
            val onSurface = colorScheme.onSurface
            
            ProfileTheme(
                backgroundBrush = Brush.verticalGradient(listOf(surface, colorScheme.surface)),
                outerBorderBrush = Brush.verticalGradient(listOf(onSurface.copy(alpha = 0.2f), onSurface.copy(alpha = 0.1f))),
                bodyOverlayColor = Color.Black.copy(alpha = 0.45f),
                contentColor = Color.White,
                cutoutColor = surface,
                pfpBorderBrush = Brush.verticalGradient(listOf(primary, primary)),
                primaryAccent = primary,
                buttonColor = primary
            )
        }
    }

    Box(
        modifier = modifier
            .then(if (showBorder) Modifier.background(theme.outerBorderBrush, RoundedCornerShape(if (isExpanded) 12.dp else 10.dp)).padding(if (isExpanded) 4.dp else 3.dp) else Modifier)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .then(if (showBorder) Modifier.clip(RoundedCornerShape(if (isExpanded) 9.dp else 7.dp)) else Modifier)
                .background(theme.backgroundBrush)
        ) {
            // Scrollable Body
            Column(modifier = Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                ProfileBanner(profile, theme, isExpanded)

                Column(modifier = Modifier.padding(start = if (isExpanded) 16.dp else 10.dp, end = 16.dp)) {
                    ProfileHeader(profile, chatState, theme, isExpanded, onExpand)
                    ProfileSections(profile, chatState, theme, isExpanded)
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
