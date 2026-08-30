package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.animation.AnimatedVisibility
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
            enter = fadeIn(tween(200, easing = LinearOutSlowInEasing)) + scaleIn(tween(200, easing = FastOutSlowInEasing), initialScale = 0.9f),
            exit = fadeOut(tween(150)) + scaleOut(tween(150), targetScale = 0.9f)
        ) {
            if (profile != null) {
                ProfileCard(
                    profile = profile,
                    isExpanded = isExpanded,
                    onExpand = { profileStore.isProfileExpanded = true },
                    onDismiss = onDismiss,
                    modifier = Modifier.then(
                        if (isExpanded) Modifier.width(800.dp).height(600.dp)
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
    presenceStore: me.lampu.lampcord.shared.state.PresenceStore = koinInject(),
    settingsStore: me.lampu.lampcord.shared.state.SettingsStore = koinInject()
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    val currentUser by userStore.currentUser.collectAsState()
    
    val decoded3y3 = remember(profile, settingsStore.profile3y3) {
        if (!settingsStore.profile3y3) return@remember null
        val bio = profile.guild_member_profile?.bio ?: profile.user_profile?.bio ?: profile.user.bio
        bio?.let { Profile3y3.decode(it) }?.let {
            try {
                kotlinx.serialization.json.Json.decodeFromString<me.lampu.lampcord.shared.model.CustomProfile>(it)
            } catch (e: Exception) { null }
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

    if (isExpanded && !isSidebar) {
        val washColor = theme.primaryAccent
        
        val bannerUrl = remember(user, guildMeta, userMeta, customProfile) {
            if (customProfile?.banner != null) {
                customProfile.banner
            } else if (guildMeta?.banner != null && profile.guild_id != null) {
                "https://cdn.discordapp.com/guilds/${profile.guild_id}/users/${user.id}/banners/${guildMeta.banner}.png?size=1024"
            } else (userMeta?.banner ?: user.banner)?.let {
                "https://cdn.discordapp.com/banners/${user.id}/$it.png?size=1024"
            }
        }

        // Deriving a themed background color for the big card
        val bigCardBg = remember(washColor) {
            val (h, s, l) = ModernProfileColors.rgbToHsl((washColor.value.toLong() shr 32).toInt())
            if (isDark) {
                // Dark mode: deep version of the accent, but more vibrant than pure black
                Color(ModernProfileColors.hslToRgb(h, s * 0.5, (l * 0.2).coerceIn(0.06, 0.12)))
            } else {
                // Light mode: soft version of the accent
                Color(ModernProfileColors.hslToRgb(h, s * 0.3, (l * 1.05).coerceIn(0.9, 0.97)))
            }
        }

        Surface(
            modifier = modifier,
            shape = outerShape,
            color = bigCardBg,
            border = BorderStroke(1.dp, Color.White.copy(alpha = 0.1f))
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                // Background Layer: Full width banner with fade
                Column(modifier = Modifier.fillMaxSize()) {
                    Box(modifier = Modifier.fillMaxWidth().height(280.dp)) {
                        if (bannerUrl != null) {
                            me.lampu.lampcord.shared.ui.components.AsyncImage(
                                model = bannerUrl,
                                contentDescription = null,
                                contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                                modifier = Modifier.fillMaxSize()
                            )
                        }
                        
                        // Heavy themed tint to make the banner subtle
                        Box(modifier = Modifier.fillMaxSize().background(washColor.copy(alpha = 0.65f)))
                        
                        // Fade to the primary theme-derived background
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(
                                    Brush.verticalGradient(
                                        0.0f to Color.Transparent,
                                        0.2f to Color.Transparent,
                                        1.0f to bigCardBg
                                    )
                                )
                        )
                    }
                    Box(modifier = Modifier.fillMaxSize().background(bigCardBg))
                }

                // Global themed wash overlay
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            Brush.verticalGradient(
                                0.0f to washColor.copy(alpha = 0.35f),
                                0.5f to washColor.copy(alpha = 0.1f),
                                1.0f to Color.Transparent
                            )
                        )
                )

                // Content Layer
                Column(modifier = Modifier.fillMaxSize()) {
                    Box(modifier = Modifier.fillMaxWidth().padding(8.dp)) {
                        IconButton(
                            onClick = { onDismiss?.invoke() },
                            modifier = Modifier.align(Alignment.TopEnd)
                        ) {
                            Icon(
                                imageVector = Icons.Filled.Close,
                                contentDescription = "Close",
                                tint = if (isDark) Color.White.copy(alpha = 0.6f) else Color.Black.copy(alpha = 0.6f)
                            )
                        }
                    }

                    Row(modifier = Modifier.fillMaxSize().padding(start = 16.dp, end = 16.dp, bottom = 16.dp)) {
                        Surface(
                            modifier = Modifier
                                .width(300.dp)
                                .fillMaxHeight()
                                .clip(innerShape),
                            color = if (isDark) Color(0xFF1E1F22) else Color(0xFFF2F3F5)
                        ) {
                            Box(modifier = Modifier.fillMaxSize().background(theme.backgroundBrush)) {
                                Column(modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                                    ProfileBanner(profile, theme, isExpanded = false, onDismiss = onDismiss, onEdit = onEditBanner, customProfileOverride = customProfile)
                                    Column(modifier = Modifier.padding(horizontal = 16.dp)) {
                                        ProfileHeader(
                                            profile = profile,
                                            theme = theme,
                                            isExpanded = true,
                                            onDismiss = onDismiss,
                                            onEditAvatar = onEditAvatar,
                                            customProfileOverride = customProfile
                                        )
                                        ProfileSections(
                                            profile = profile,
                                            theme = theme,
                                            isExpanded = true,
                                            showMemberSince = showMemberSince
                                        )
                                    }
                                }
                            }
                        }

                        Spacer(Modifier.width(24.dp))

                        Column(modifier = Modifier.weight(1f)) {
                            var selectedTab by remember { mutableStateOf(0) }
                            val tabs = listOf("Board", "Activity", "Mutual Friends", "Mutual Servers")
                            
                            ScrollableTabRow(
                                selectedTabIndex = selectedTab,
                                containerColor = Color.Transparent,
                                divider = {},
                                edgePadding = 0.dp,
                                indicator = { tabPositions ->
                                    if (selectedTab < tabPositions.size) {
                                        TabRowDefaults.SecondaryIndicator(
                                            modifier = Modifier.tabIndicatorOffset(tabPositions[selectedTab]),
                                            color = if (isDark) Color.White else Color.Black
                                        )
                                    }
                                }
                            ) {
                                tabs.forEachIndexed { index, title ->
                                    Tab(
                                        selected = selectedTab == index,
                                        onClick = { selectedTab = index },
                                        text = { 
                                            Text(
                                                text = title,
                                                style = MaterialTheme.typography.titleSmall,
                                                fontWeight = if (selectedTab == index) FontWeight.Bold else FontWeight.Normal,
                                                color = if (selectedTab == index) {
                                                    if (isDark) Color.White else Color.Black
                                                } else {
                                                    if (isDark) Color.White.copy(alpha = 0.6f) else Color.Black.copy(alpha = 0.6f)
                                                }
                                            ) 
                                        }
                                    )
                                }
                            }

                            Spacer(Modifier.height(24.dp))

                            Box(modifier = Modifier.fillMaxSize().padding(top = 8.dp)) {
                                when (selectedTab) {
                                    0 -> { // Board
                                        Column(
                                            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()),
                                            verticalArrangement = Arrangement.spacedBy(16.dp)
                                        ) {
                                            ModernProfileBoardCard(title = "Favorite Game", isDark = isDark) {
                                                Row(verticalAlignment = Alignment.CenterVertically) {
                                                    Box(Modifier.size(64.dp).background(Color.Gray.copy(alpha = 0.2f), RoundedCornerShape(8.dp)))
                                                    Spacer(Modifier.width(12.dp))
                                                    Column {
                                                        Text("Game Title", style = MaterialTheme.typography.titleMedium, color = if (isDark) Color.White else Color.Black)
                                                        Text("Playing for 2 hours", style = MaterialTheme.typography.bodySmall, color = (if (isDark) Color.White else Color.Black).copy(alpha = 0.6f))
                                                    }
                                                }
                                            }
                                            
                                            ModernProfileBoardCard(title = "Games I Like", isDark = isDark) {
                                                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                                    repeat(3) {
                                                        Box(Modifier.size(64.dp, 80.dp).background(Color.Gray.copy(alpha = 0.2f), RoundedCornerShape(8.dp)))
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    1 -> { // Activity
                                        val presences by presenceStore.presences.collectAsState()
                                        val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]
                                        val activities = (profile.activities.ifEmpty { presence?.activities ?: emptyList() }).filter { it.type != 4 }
                                        
                                        Column(
                                            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()),
                                            verticalArrangement = Arrangement.spacedBy(24.dp)
                                        ) {
                                            if (activities.isNotEmpty()) {
                                                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                                    Text(
                                                        "Current activity",
                                                        style = MaterialTheme.typography.labelMedium,
                                                        fontWeight = FontWeight.Bold,
                                                        color = (if (isDark) Color.White else Color.Black).copy(alpha = 0.6f)
                                                    )
                                                    activities.forEach { activity ->
                                                        me.lampu.lampcord.shared.ui.components.UserActivity(
                                                            activity = activity,
                                                            compact = false
                                                        )
                                                    }
                                                }
                                            }
                                            
                                            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                                Text(
                                                    "Recent activity",
                                                    style = MaterialTheme.typography.labelMedium,
                                                    fontWeight = FontWeight.Bold,
                                                    color = (if (isDark) Color.White else Color.Black).copy(alpha = 0.6f)
                                                )
                                                
                                                // Placeholder for actual activity history
                                                repeat(2) {
                                                    Surface(
                                                        modifier = Modifier.fillMaxWidth(),
                                                        shape = RoundedCornerShape(12.dp),
                                                        color = if (isDark) Color(0xFF2B2D31).copy(alpha = 0.5f) else Color(0xFFE3E5E8).copy(alpha = 0.5f)
                                                    ) {
                                                        Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
                                                            Box(Modifier.size(48.dp).background(Color.Gray.copy(alpha = 0.2f), RoundedCornerShape(8.dp)))
                                                            Spacer(Modifier.width(12.dp))
                                                            Column {
                                                                Text("Past Activity", style = MaterialTheme.typography.titleSmall, color = if (isDark) Color.White else Color.Black)
                                                                Text("Played 3 days ago", style = MaterialTheme.typography.bodySmall, color = (if (isDark) Color.White else Color.Black).copy(alpha = 0.6f))
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    else -> {
                                        Text(
                                            text = "${tabs[selectedTab]} Content Coming Soon",
                                            modifier = Modifier.align(Alignment.Center),
                                            style = MaterialTheme.typography.bodyLarge,
                                            color = (if (isDark) Color.White else Color.Black).copy(alpha = 0.4f)
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        Box(
            modifier = modifier
                .then(if (showBorder) Modifier.background(theme.outerBorderBrush, outerShape).padding(4.dp) else Modifier)
        ) {
            Column(
                modifier = Modifier
                    .then(if (isSidebar) Modifier.fillMaxSize() else Modifier.fillMaxWidth().wrapContentHeight())
                    .then(if (showBorder) Modifier.clip(innerShape) else Modifier)
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
}

@Composable
private fun ModernProfileBoardCard(
    title: String,
    isDark: Boolean = true,
    content: @Composable () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = if (isDark) Color(0xFF2B2D31) else Color(0xFFE3E5E8),
        shape = RoundedCornerShape(12.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    color = if (isDark) Color.White else Color.Black
                )
                Icon(
                    imageVector = Icons.Filled.MoreHoriz,
                    contentDescription = null,
                    tint = (if (isDark) Color.White else Color.Black).copy(alpha = 0.6f),
                    modifier = Modifier.size(20.dp)
                )
            }
            content()
        }
    }
}
