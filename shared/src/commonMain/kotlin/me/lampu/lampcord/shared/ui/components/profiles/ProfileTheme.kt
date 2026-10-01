package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import me.lampu.lampcord.shared.model.CustomProfile
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ClientProfileStore
import me.lampu.lampcord.shared.state.SettingsStore
import com.materialkolor.PaletteStyle
import me.lampu.lampcord.shared.ui.theme.rememberPlatformColorScheme
import me.lampu.lampcord.shared.utils.Logging

/** The resolved theme for a profile, plus the custom profile it was derived from. */
internal data class ResolvedProfileTheme(
    val theme: ProfileTheme,
    val customProfile: CustomProfile?,
)

/**
 * Resolves the colours a profile is themed from.
 *
 * The compact card and the full desktop view both need this so the view's backdrop can fade into
 * exactly the colours the card itself uses.
 */
@Composable
internal fun rememberProfileTheme(
    profile: UserProfile,
    customProfileOverride: CustomProfile?,
    clientProfileStore: ClientProfileStore,
    settingsStore: SettingsStore,
): ResolvedProfileTheme {
    val colorScheme = MaterialTheme.colorScheme
        val user = profile.user
        val guildMeta = profile.guild_member_profile
        val userMeta = profile.user_profile
    
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
        val localOverrides by clientProfileStore.localOverrides.collectAsState()
        val localBanner = localOverrides[user.id]?.banner?.takeIf { it.isNotEmpty() }
        val localAvatar = localOverrides[user.id]?.avatar?.takeIf { it.isNotEmpty() }
        val dbProfile = remember(user.id, settingsStore.userBg, settingsStore.userPfp, customProfiles, localOverrides) {
            clientProfileStore.getCustomProfile(user.id)
        }
        val customProfile = customProfileOverride ?: remember(decoded3y3, dbProfile, localBanner, localAvatar) {
            val merged = if (decoded3y3 != null) {
                decoded3y3.copy(
                    banner = decoded3y3.banner ?: dbProfile?.banner,
                    avatar = decoded3y3.avatar ?: dbProfile?.avatar,
                    theme_colors = decoded3y3.theme_colors ?: dbProfile?.theme_colors,
                    accent_color = decoded3y3.accent_color ?: dbProfile?.accent_color
                )
            } else {
                dbProfile
            }
            merged?.copy(
                banner = localBanner ?: if (settingsStore.userBg) merged.banner else null,
                avatar = localAvatar ?: if (settingsStore.userPfp) merged.avatar else null
            )
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
                    customTextColor = if (isLightMode) Color.Black else Color.White,
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
                    buttonTextColor = profileScheme.onPrimary,
                )
            }
        }
    return ResolvedProfileTheme(theme, customProfile)
}
