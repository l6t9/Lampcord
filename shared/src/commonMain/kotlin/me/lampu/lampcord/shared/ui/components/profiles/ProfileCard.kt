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
import androidx.compose.ui.graphics.luminance
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
import me.lampu.lampcord.shared.ui.theme.rememberPlatformColorScheme
import com.materialkolor.PaletteStyle

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
    isSidebar: Boolean = false,
    showMemberSince: Boolean = false,
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
                val c2 = ModernProfileColors.mix(c, 0xFFFFFFFF.toInt(), 0.12)
                listOf(c, c2)
            }
        }
    }

    val colorScheme = MaterialTheme.colorScheme
    val isDark = colorScheme.surface.luminance() < 0.5f
    
    val profileSeed = remember(themeColors) {
        themeColors?.get(0)?.let { Color(it or 0xFF000000.toInt()) } ?: colorScheme.primary
    }

    val profileScheme = rememberPlatformColorScheme(
        seedColor = profileSeed,
        isDark = isDark,
        paletteStyle = PaletteStyle.TonalSpot,
        useMaterialYou = false
    )

    val theme = remember(themeColors, colorScheme, profileScheme) {
        if (themeColors != null && themeColors.size >= 2) {
            val primary = themeColors[0]
            val accent = themeColors[1]
            
            val (h1, s1, l1) = ModernProfileColors.rgbToHsl(primary)
            val (h2, s2, l2) = ModernProfileColors.rgbToHsl(accent)

            val base1Lum = ModernProfileColors.getLuminance(primary)
            val base2Lum = ModernProfileColors.getLuminance(accent)
            val isLightMode = base1Lum > 0.7 && base2Lum > 0.75

            // 1. Background Gradient (Darkened strictly from HSL for saturation and depth)
            val bg1 = if (base1Lum > 0.4) {
                Color(ModernProfileColors.hslToRgb(h1, s1 * 0.8, (l1 * 0.88).coerceIn(0.0, 1.0)))
            } else {
                Color(ModernProfileColors.hslToRgb(h1, s1, (l1 * 0.65).coerceIn(0.0, 1.0)))
            }

            val bg2 = if (base2Lum > 0.4) {
                Color(ModernProfileColors.hslToRgb(h2, s2 * 0.8, (l2 * 1.05).coerceIn(0.0, 1.0)))
            } else {
                Color(ModernProfileColors.hslToRgb(h2, s2, (l2 * 0.65).coerceIn(0.0, 1.0)))
            }
            
            // 2. Outer Border (Lighter and more saturated versions for that "Accent" look)
            val b1 = if (isLightMode) {
                Color(ModernProfileColors.hslToRgb(h1, (s1 * 1.15).coerceIn(0.0, 1.0), (l1 * 0.8).coerceIn(0.0, 1.0)))
            } else {
                Color(ModernProfileColors.hslToRgb(h1, (s1 * 1.15).coerceIn(0.0, 1.0), (l1 * 1.3).coerceIn(0.0, 1.0)))
            }
            val b2 = if (isLightMode) {
                Color(ModernProfileColors.hslToRgb(h2, (s2 * 1.15).coerceIn(0.0, 1.0), (l2 * 0.8).coerceIn(0.0, 1.0)))
            } else {
                Color(ModernProfileColors.hslToRgb(h2, (s2 * 1.15).coerceIn(0.0, 1.0), (l2 * 1.3).coerceIn(0.0, 1.0)))
            }

            val avgLum = (base1Lum + base2Lum) / 2.0
            val bodyOverlayColor = if (isLightMode) {
                when {
                    avgLum >= 0.85 -> Color.White.copy(alpha = 0.0f)
                    else -> Color.White.copy(alpha = 0.1f)
                }
            } else {
                Color.Black.copy(alpha = 0.45f)
            }

            val cardL = if (isLightMode) {
                l1
            } else {
                // Dark mode: Ensure cards are significantly darker than the background
                // We use a much lower factor than the background (which uses 0.65-0.88)
                (l1 * 0.12).coerceIn(0.01, 0.1)
            }
            val cardS = (s1 * 0.85).coerceIn(0.0, 1.0)
            val cardColor = Color(ModernProfileColors.hslToRgb(h1, cardS, cardL))
            
            ProfileTheme(
                backgroundBrush = Brush.verticalGradient(0.0f to bg1, 0.35f to bg1, 1.0f to bg2),
                outerBorderBrush = Brush.verticalGradient(listOf(b1, b2)),
                bodyOverlayColor = bodyOverlayColor,
                cardColor = cardColor,
                tagColor = profileSeed.copy(alpha = 0.4f),
                contentColor = if (isLightMode) Color.Black else Color.White,
                cutoutColor = bg1,
                pfpBorderBrush = Brush.verticalGradient(listOf(Color(primary or 0xFF000000.toInt()), Color(accent or 0xFF000000.toInt()))),
                primaryAccent = Color(primary or 0xFF000000.toInt()),
                buttonColor = profileScheme.secondary,
                buttonTextColor = profileScheme.onSecondary,
                isCustom = true,
                themeColors = listOf(Color(primary or 0xFF000000.toInt()), Color(accent or 0xFF000000.toInt()))
            )
        } else {
            val primary = colorScheme.primary
            val surface = colorScheme.surfaceContainer
            val onSurface = colorScheme.onSurface
            
            ProfileTheme(
                backgroundBrush = Brush.verticalGradient(listOf(surface, colorScheme.surface)),
                outerBorderBrush = Brush.verticalGradient(listOf(onSurface.copy(alpha = 0.2f), onSurface.copy(alpha = 0.1f))),
                bodyOverlayColor = Color.Black.copy(alpha = 0.45f),
                cardColor = profileScheme.surfaceContainerHigh,
                tagColor = profileSeed.copy(alpha = 0.4f),
                contentColor = Color.White,
                cutoutColor = surface,
                pfpBorderBrush = Brush.verticalGradient(listOf(primary, primary)),
                primaryAccent = primary,
                buttonColor = profileScheme.primary,
                buttonTextColor = profileScheme.onPrimary
            )
        }
    }

    val outerShape = MaterialTheme.shapes.large
    val innerShape = MaterialTheme.shapes.medium

    Box(
        modifier = modifier
            .then(if (showBorder) Modifier.background(theme.outerBorderBrush, outerShape).padding(if (isExpanded || isSidebar) 8.dp else 4.dp) else Modifier)
    ) {
        Column(
            modifier = Modifier
                .then(if (isSidebar) Modifier.fillMaxSize() else Modifier.fillMaxWidth().wrapContentHeight())
                .then(if (showBorder) Modifier.clip(innerShape) else Modifier)
                .background(theme.backgroundBrush)
        ) {
            // Scrollable Body
            Column(
                modifier = Modifier
                    .then(if (isSidebar) Modifier.weight(1f) else Modifier.wrapContentHeight())
                    .verticalScroll(rememberScrollState())
            ) {
                ProfileBanner(profile, theme, isExpanded)

                Column(modifier = Modifier.padding(start = if (isExpanded) 16.dp else 10.dp, end = 16.dp)) {
                    ProfileHeader(profile, chatState, theme, isExpanded, onExpand)
                    ProfileSections(profile, chatState, theme, isExpanded, showMemberSince)
                }
            }

            if (!isExpanded && user.id != chatState.currentUser?.id) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(start = 16.dp, end = 16.dp, bottom = 12.dp)
                ) {
                    Button(
                        onClick = { onExpand?.invoke() },
                        modifier = Modifier.fillMaxWidth().height(32.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = theme.buttonColor,
                            contentColor = theme.buttonTextColor
                        ),
                        shape = CircleShape,
                        contentPadding = PaddingValues(horizontal = 12.dp)
                    ) {
                        Text("View Full Profile", style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Medium)
                    }
                }
            } else {
                Spacer(Modifier.height(16.dp))
            }
        }
    }
}
