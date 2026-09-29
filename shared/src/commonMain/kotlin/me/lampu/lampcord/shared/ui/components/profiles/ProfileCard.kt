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
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.materialkolor.PaletteStyle
import me.lampu.lampcord.shared.model.ProfileCollectibles
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ClientProfileStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.theme.rememberPlatformColorScheme
import me.lampu.lampcord.shared.utils.Logging

import org.koin.compose.koinInject
import androidx.compose.material3.LocalContentColor

import me.lampu.lampcord.shared.ui.components.ShimmerBox
import androidx.compose.ui.zIndex

@Composable
fun ProfileCardSkeleton(
    modifier: Modifier = Modifier,
    isExpanded: Boolean = false,
    fillAvailableHeight: Boolean = false,
) {
    val colorScheme = MaterialTheme.colorScheme
    val surface = colorScheme.surfaceContainer
    val onSurface = colorScheme.onSurface
    
    val theme = remember(colorScheme) {
        ProfileTheme(
            backgroundBrush = Brush.verticalGradient(listOf(surface, colorScheme.surface)),
            outerBorderBrush = Brush.verticalGradient(listOf(onSurface.copy(alpha = 0.2f), onSurface.copy(alpha = 0.1f))),
            bodyOverlayColor = Color.Black.copy(alpha = 0.45f),
            cardColor = colorScheme.surfaceContainerHigh,
            tagColor = onSurface.copy(alpha = 0.1f),
            contentColor = onSurface,
            cutoutColor = surface,
            pfpBorderBrush = Brush.verticalGradient(listOf(colorScheme.outline, colorScheme.outline)),
            primaryAccent = colorScheme.primary,
            buttonColor = colorScheme.primary,
            buttonTextColor = colorScheme.onPrimary
        )
    }

    val sheetShape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp)
    val expandedShape = RoundedCornerShape(0.dp)

    Box(
        modifier = modifier
    ) {
        Column(
            modifier = Modifier
                .then(if (fillAvailableHeight) Modifier.fillMaxWidth().fillMaxHeight() else Modifier.fillMaxWidth().wrapContentHeight())
                .clip(if (isExpanded) expandedShape else sheetShape)
                .background(theme.backgroundBrush)
        ) {
            Column(
                modifier = Modifier
                    .then(if (fillAvailableHeight) Modifier.weight(1f) else Modifier.wrapContentHeight())
                    .verticalScroll(rememberScrollState())
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(if (isExpanded) 160.dp else 105.dp)
                        .background(colorScheme.surfaceContainerHighest)
                )

                Column(modifier = Modifier.padding(start = if (isExpanded) 16.dp else 10.dp, end = 16.dp)) {
                    Row(
                        verticalAlignment = Alignment.Top,
                        modifier = Modifier.fillMaxWidth().padding(end = 16.dp).zIndex(1f)
                    ) {
                        Box(
                            modifier = Modifier
                                .offset(y = (-45).dp)
                                .size(94.dp)
                                .background(theme.cutoutColor, CircleShape)
                                .padding(6.dp)
                        ) {
                            ShimmerBox(
                                modifier = Modifier.fillMaxSize(),
                                shape = CircleShape
                            )
                        }
                    }

                    Column(modifier = Modifier.offset(y = (-35).dp)) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            ShimmerBox(
                                modifier = Modifier.width(150.dp).height(24.dp),
                                shape = RoundedCornerShape(12.dp)
                            )
                        }
                        Spacer(Modifier.height(4.dp))
                        ShimmerBox(
                            modifier = Modifier.width(100.dp).height(16.dp),
                            shape = RoundedCornerShape(8.dp)
                        )
                        
                        Spacer(Modifier.height(8.dp))
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            repeat(3) {
                                ShimmerBox(modifier = Modifier.size(22.dp), shape = RoundedCornerShape(4.dp))
                            }
                        }

                        Spacer(Modifier.height(24.dp))
                        repeat(2) {
                            ShimmerBox(
                                modifier = Modifier.fillMaxWidth().height(60.dp),
                                shape = RoundedCornerShape(12.dp)
                            )
                            Spacer(Modifier.height(8.dp))
                        }
                    }
                }
            }
            Spacer(Modifier.height(16.dp))
        }
    }
}

@Composable
fun UserProfileDialog(
    profile: UserProfile?,
    onDismiss: () -> Unit,
    profileStore: ProfileStore = koinInject()
) {
    val popupPosition = profileStore.profilePosition
    val isExpanded = profileStore.isProfileExpanded
    val cardWidth = if (isExpanded) 450.dp else 300.dp
    val frameInsets = rememberFrameInsets(profile, profileStore, cardWidth)
    val density = LocalDensity.current
    val frameInsetPx = with(density) {
        IntOffset(
            x = frameInsets.horizontal.roundToPx(),
            y = frameInsets.top.roundToPx()
        )
    }

    val anchor = popupPosition
    val isCentered = anchor == null || isExpanded
    val placementOffset = if (anchor == null || isExpanded) {
        IntOffset.Zero
    } else {
        IntOffset(
            x = (if (anchor.x < 500) (anchor.x + 60).toInt() else (anchor.x - 320).toInt()) - frameInsetPx.x,
            y = ((anchor.y.toInt() - 100) - frameInsetPx.y).coerceAtLeast(10)
        )
    }

    Box(modifier = Modifier.fillMaxSize()) {
        if (profile != null) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .combinedClickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null,
                        onClick = onDismiss,
                        onLongClick = {}
                    )
            )
        }

        AnimatedVisibility(
            visible = profile != null,
            modifier = Modifier
                .align(if (isCentered) Alignment.Center else Alignment.TopStart)
                .offset { placementOffset },
            enter = if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) EnterTransition.None else fadeIn(tween(200, easing = LinearOutSlowInEasing)) + scaleIn(tween(200, easing = FastOutSlowInEasing), initialScale = 0.9f),
            exit = if (me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) ExitTransition.None else fadeOut(tween(150)) + scaleOut(tween(150), targetScale = 0.9f)
        ) {
            if (profile != null) {
                ProfileCard(
                    profile = profile,
                    isExpanded = isExpanded,
                    onExpand = { profileStore.isProfileExpanded = true },
                    onDismiss = onDismiss,
                    modifier = Modifier
                        .padding(
                            start = frameInsets.horizontal,
                            top = frameInsets.top,
                            end = frameInsets.horizontal,
                            bottom = frameInsets.bottom
                        )
                        .then(
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
    fillAvailableHeight: Boolean = false,
    showMemberSince: Boolean = false,
    showMutualsInConnections: Boolean = true,
    onExpand: (() -> Unit)? = null,
    onDismiss: (() -> Unit)? = null,
    onEditBanner: (() -> Unit)? = null,
    onEditAvatar: (() -> Unit)? = null,
    customProfileOverride: me.lampu.lampcord.shared.model.CustomProfile? = null,
    showBorder: Boolean = true,
    scrollState: androidx.compose.foundation.ScrollState? = null,
    avatarScale: Float = 1f,
    bannerHeightOverride: Dp? = null,
    topShape: Shape? = null,
    bannerHeightRatio: Float = 0f,
    bannerContentScale: ContentScale = ContentScale.Crop,
    userStore: UserStore = koinInject(),
    clientProfileStore: ClientProfileStore = koinInject(),
    settingsStore: me.lampu.lampcord.shared.state.SettingsStore = koinInject(),
    profileStore: me.lampu.lampcord.shared.state.ProfileStore = koinInject()
) {
    val resolved = rememberProfileTheme(profile, customProfileOverride, clientProfileStore, settingsStore)
    val theme = resolved.theme
    val customProfile = resolved.customProfile

    val fallbackScrollState = rememberScrollState()

    val outerShape = RoundedCornerShape(16.dp)
    val innerShape = RoundedCornerShape(12.dp)
    val sheetShape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp)
    val expandedShape = RoundedCornerShape(0.dp)

    BoxWithConstraints(
        modifier = modifier
            .then(if (showBorder) Modifier.background(theme.outerBorderBrush, outerShape).padding(4.dp) else Modifier)
    ) {
        val railTop = profileBannerHeight(maxWidth, isExpanded, bannerHeightOverride, bannerHeightRatio)
        val effectSku = remember(profile) {
            ProfileCollectibles.effectSku(profile.user_profile, profile.guild_member_profile)
        }
        val frameSku = remember(profile) {
            ProfileCollectibles.frameSku(profile.user_profile, profile.guild_member_profile)
        }
        val effectProduct = effectSku?.let { profileStore.getEffect(it) }
        val frameProduct = frameSku?.let { profileStore.getFrame(it) }

        Column(
            modifier = Modifier
                .then(if (isSidebar) Modifier.fillMaxSize() else if (fillAvailableHeight) Modifier.fillMaxWidth().fillMaxHeight() else Modifier.fillMaxWidth().wrapContentHeight())
                .then(
                    if (showBorder) Modifier.clip(innerShape)
                    else if (!isSidebar) Modifier.clip(
                        when {
                            isExpanded && topShape != null -> topShape
                            isExpanded -> expandedShape
                            else -> sheetShape
                        }
                    )
                    else Modifier
                )
                .background(theme.backgroundBrush)
        ) {
            CompositionLocalProvider(LocalContentColor provides theme.contentColor) {
                Column(
                    modifier = Modifier
                        .then(if (isSidebar || fillAvailableHeight) Modifier.weight(1f) else Modifier.wrapContentHeight())
                        .verticalScroll(scrollState ?: fallbackScrollState)
                ) {
                    ProfileBanner(profile, theme, isExpanded, bannerHeightOverride, topShape, bannerHeightRatio, bannerContentScale, onDismiss = onDismiss, onEdit = onEditBanner, customProfileOverride = customProfile)

                    Column(modifier = Modifier.padding(start = if (isExpanded) { 16.dp } else 10.dp, end = 16.dp)) {
                        ProfileHeader(
                            profile = profile,
                            theme = theme,
                            isExpanded = isExpanded,
                            avatarScale = avatarScale,
                            onExpand = onExpand,
                            onDismiss = onDismiss,
                            onEditAvatar = onEditAvatar,
                            customProfileOverride = customProfile
                        )
                        ProfileSections(
                            profile = profile,
                            theme = theme,
                            isExpanded = isExpanded,
                            showMemberSince = showMemberSince,
                            showMutualsInConnections = showMutualsInConnections
                        )
                    }
                }
            }

            if (!fillAvailableHeight) {
                Spacer(Modifier.height(16.dp))
            }
        }

        if (effectProduct != null) {
            ProfileEffectOverlay(
                product = effectProduct,
                modifier = Modifier.matchParentSize(),
                animate = !Settings.shared.reduceMotion
            )
        }

        if (frameProduct != null) {
            ProfileFrameOverlay(
                product = frameProduct,
                modifier = Modifier.matchParentSize(),
                railTop = railTop
            )
        }
    }
}
