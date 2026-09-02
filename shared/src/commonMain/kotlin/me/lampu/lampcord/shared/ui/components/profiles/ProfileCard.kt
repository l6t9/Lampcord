package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.material3.TabRowDefaults.tabIndicatorOffset
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import com.materialkolor.PaletteStyle
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ClientProfileStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.theme.rememberPlatformColorScheme
import me.lampu.lampcord.shared.utils.Logging
import org.koin.compose.koinInject
import androidx.compose.material3.LocalContentColor

@Composable
fun UserProfileDialog(
    profile: UserProfile?,
    onDismiss: () -> Unit,
    profileStore: ProfileStore = koinInject()
) {
    val popupPosition = profileStore.profilePosition
    val isExpanded = profileStore.isProfileExpanded

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
            enter = if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) EnterTransition.None else fadeIn(tween(200, easing = LinearOutSlowInEasing)) + scaleIn(tween(200, easing = FastOutSlowInEasing), initialScale = 0.9f),
            exit = if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) ExitTransition.None else fadeOut(tween(150)) + scaleOut(tween(150), targetScale = 0.9f)
        ) {
            if (profile != null) {
                ProfileCard(
                    profile = profile,
                    isExpanded = isExpanded,
                    onExpand = { profileStore.isProfileExpanded = true },
                    onDismiss = onDismiss,
                    modifier = Modifier.then(
                        if (isExpanded) Modifier.width(450.dp).heightIn(max = 800.dp)
                        else Modifier.width(300.dp).wrapContentHeight()
                    )
                )
            }
        }
    }
}

@Composable
fun ProfileCard(
    profile: UserProfile, 
    modifier: Modifier = Modifier,
    isExpanded: Boolean = false,
    isSidebar: Boolean = false,
    showMemberSince: Boolean = false,
    onExpand: (() -> Unit)? = null,
    onDismiss: (() -> Unit)? = null,
    onEditBanner: (() -> Unit)? = null,
    onEditAvatar: (() -> Unit)? = null,
    customProfileOverride: me.lampu.lampcord.shared.model.CustomProfile? = null,
    showBorder: Boolean = true,
    userStore: UserStore = koinInject(),
    clientProfileStore: ClientProfileStore = koinInject(),
    settingsStore: me.lampu.lampcord.shared.state.SettingsStore = koinInject()
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    val currentUser by userStore.currentUser.collectAsState()
    
    val decoded3y3 = remember(profile, settingsStore.profile3y3) {
        if (!settingsStore.profile3y3) return@remember null
        val bio = profile.guild_member_profile?.bio ?: profile.user_profile?.bio ?: profile.user.bio
        bio?.let { 
            val decoded = Profile3y3.decode(it)
            if (decoded != null) {
                Logging.d("Profile3y3", "Decoded 3y3 for ${user.username}: $decoded")
            }
            decoded
        }?.let {
            try {
                kotlinx.serialization.json.Json.decodeFromString<me.lampu.lampcord.shared.model.CustomProfile>(it)
            } catch (e: Exception) { 
                Logging.e("Profile3y3", "Failed to parse 3y3 JSON for ${user.username}", e)
                null 
            }
        }
    }
    
    val customProfiles by clientProfileStore.customProfiles.collectAsState()
    val dbProfile = remember(user.id, settingsStore.userBg, settingsStore.userPfp, customProfiles) { 
        clientProfileStore.getCustomProfile(user.id)?.let {
            var updated = it
            if (!settingsStore.userBg) updated = updated.copy(banner = null)
            if (!settingsStore.userPfp) updated = updated.copy(avatar = null)
            updated
        }
    }

    val customProfile = customProfileOverride ?: remember(decoded3y3, dbProfile) {
        if (decoded3y3 != null) {
            decoded3y3.copy(
                banner = decoded3y3.banner ?: dbProfile?.banner,
                avatar = decoded3y3.avatar ?: dbProfile?.avatar,
                theme_colors = decoded3y3.theme_colors ?: dbProfile?.theme_colors,
                accent_color = decoded3y3.accent_color ?: dbProfile?.accent_color
            )
        } else {
            dbProfile
        }
    }

    val themeColors = remember(profile, customProfile) {
        val rawColors = customProfile?.theme_colors ?: guildMeta?.theme_colors ?: userMeta?.theme_colors
        val base = if (!rawColors.isNullOrEmpty()) rawColors else {
            val accent = customProfile?.accent_color ?: guildMeta?.accent_color ?: userMeta?.accent_color ?: user.accent_color
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
                contentColor = profileScheme.onSurface,
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
                contentColor = profileScheme.onSurface,
                cutoutColor = surface,
                pfpBorderBrush = Brush.verticalGradient(listOf(primary, primary)),
                primaryAccent = primary,
                buttonColor = profileScheme.primary,
                buttonTextColor = profileScheme.onPrimary
            )
        }
    }

    val outerShape = RoundedCornerShape(16.dp)
    val innerShape = RoundedCornerShape(12.dp)
    val sheetShape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp)

    Box(
        modifier = modifier
            .then(if (showBorder) Modifier.background(theme.outerBorderBrush, outerShape).padding(4.dp) else Modifier)
    ) {
        Column(
            modifier = Modifier
                .then(if (isSidebar) Modifier.fillMaxSize() else Modifier.fillMaxWidth().wrapContentHeight())
                .then(
                    if (showBorder) Modifier.clip(innerShape) 
                    else if (!isSidebar) Modifier.clip(sheetShape)
                    else Modifier
                )
                .background(theme.backgroundBrush)
        ) {
            CompositionLocalProvider(LocalContentColor provides theme.contentColor) {
                Column(
                    modifier = Modifier
                        .then(if (isSidebar) Modifier.weight(1f) else Modifier.wrapContentHeight())
                        .verticalScroll(rememberScrollState())
                ) {
                    ProfileBanner(profile, theme, isExpanded, onDismiss = onDismiss, onEdit = onEditBanner, customProfileOverride = customProfile)

                    Column(modifier = Modifier.padding(start = if (isExpanded) { 16.dp } else 10.dp, end = 16.dp)) {
                        ProfileHeader(
                            profile = profile,
                            theme = theme,
                            isExpanded = isExpanded,
                            onExpand = onExpand,
                            onDismiss = onDismiss,
                            onEditAvatar = onEditAvatar,
                            customProfileOverride = customProfile
                        )
                        ProfileSections(
                            profile = profile,
                            theme = theme,
                            isExpanded = isExpanded,
                            showMemberSince = showMemberSince
                        )
                    }
                }
            }

            if (!isExpanded && user.id != currentUser?.id) {
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
