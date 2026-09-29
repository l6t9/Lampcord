package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ClientProfileStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ImageLoadState
import me.lampu.lampcord.shared.ui.components.VerticalScrollbar
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.combinedClickableCursor
import org.koin.compose.koinInject

private val CARD_WIDTH = 360.dp
private val AVATAR_SCALE = 1.45f
private const val BANNER_RATIO = 140f / 340f
private val CORNER = 16.dp
private val PANEL_MARGIN_H = 40.dp
private val PANEL_MARGIN_V = 32.dp

@Composable
fun FullProfileDialog(
    profile: UserProfile,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    clientProfileStore: ClientProfileStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
) {
    val scrollState = rememberScrollState()
    val resolved = rememberProfileTheme(profile, null, clientProfileStore, settingsStore)
    val theme = resolved.theme
    val customProfile = resolved.customProfile

    val frameInsets = rememberFrameInsets(profile, profileStore, CARD_WIDTH)
    val cardInset = 8.dp

    Box(
        modifier = modifier
            .fillMaxSize()
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown && event.key == Key.Escape) {
                    onDismiss()
                    true
                } else {
                    false
                }
            }
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .combinedClickableCursor(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = onDismiss,
                    onLongClick = {}
                )
        )

        Box(
            modifier = Modifier
                .align(Alignment.Center)
                .padding(
                    start = frameInsets.horizontal,
                    top = frameInsets.top,
                    end = frameInsets.horizontal,
                    bottom = frameInsets.bottom
                )
        ) {
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .clip(MaterialTheme.shapes.large)
            ) {
                ProfileBackdrop(profile, customProfile?.banner, theme)
            }

            Box(
                modifier = Modifier
                    .padding(start = PANEL_MARGIN_H, top = PANEL_MARGIN_V, end = PANEL_MARGIN_H, bottom = 0.dp)
                    .widthIn(max = 1200.dp)
                    .fillMaxWidth(0.95f)
                    .heightIn(max = 850.dp)
                    .fillMaxHeight(0.9f)
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(MaterialTheme.shapes.large)
                        .combinedClickableCursor(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                            onClick = onDismiss,
                            onLongClick = {}
                        )
                )

                Row(
                    modifier = Modifier.fillMaxSize(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .width(CARD_WIDTH)
                            .fillMaxHeight()
                            .padding(start = cardInset, top = cardInset)
                    ) {
                        ProfileCard(
                            profile = profile,
                            modifier = Modifier.fillMaxSize(),
                            isExpanded = true,
                            fillAvailableHeight = true,
                            showBorder = false,
                            avatarScale = AVATAR_SCALE,
                            topShape = RoundedCornerShape(topStart = CORNER, topEnd = CORNER),
                            bannerContentScale = ContentScale.Crop,
                            bannerHeightRatio = BANNER_RATIO,
                            customProfileOverride = customProfile,
                            scrollState = scrollState,
                            showMutualsInConnections = false,
                            onDismiss = onDismiss
                        )
                        VerticalScrollbar(
                            state = scrollState,
                            modifier = Modifier
                                .align(Alignment.CenterEnd)
                                .fillMaxHeight()
                                .padding(end = 2.dp),
                            isVisible = scrollState.value > 0 && scrollState.value < scrollState.maxValue
                        )
                    }

                    FullProfileDetails(
                        profile = profile,
                        theme = theme,
                        topShape = RoundedCornerShape(topStart = CORNER, topEnd = CORNER),
                        onOpenProfile = { userId -> profileStore.showProfile(userId) },
                        onOpenGuild = { guildId ->
                            guildStore.guilds.value.find { it.id == guildId }?.let {
                                navigationStore.selectedGuild = it
                            }
                            profileStore.closeFullProfile()
                        },
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight()
                            .padding(top = cardInset, end = cardInset)
                    )
                }
            }

            Surface(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(top = 14.dp, end = 14.dp)
                    .size(32.dp),
                shape = CircleShape,
                color = theme.cardColor
            ) {
                IconButton(onClick = onDismiss) {
                    Icon(
                        imageVector = Icons.Filled.Close,
                        contentDescription = "Close",
                        tint = Color.White,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun ProfileBackdrop(
    profile: UserProfile,
    customBanner: String?,
    theme: ProfileTheme,
) {
    val user = profile.user
    val guildId = profile.guild_id
    val memberBanner = profile.guild_member_profile?.banner
    val bannerUrl = when {
        !customBanner.isNullOrEmpty() -> customBanner
        guildId != null && memberBanner != null ->
            "https://cdn.discordapp.com/guilds/$guildId/users/${user.id}/banners/$memberBanner.png?size=1024"
        !user.banner.isNullOrEmpty() ->
            "https://cdn.discordapp.com/banners/${user.id}/${user.banner}.png?size=1024"
        else -> null
    }

    var loaded by remember(bannerUrl) { mutableStateOf(bannerUrl == null) }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(theme.backgroundBrush)
    ) {
        if (bannerUrl != null) {
            AsyncImage(
                model = bannerUrl,
                contentDescription = null,
                modifier = Modifier
                    .fillMaxSize()
                    .blur(48.dp)
                    .scale(1.25f)
                    .graphicsLayer { alpha = if (loaded) 1f else 0f },
                contentScale = ContentScale.Crop,
                showPlaceholder = false,
                onState = { if (it is ImageLoadState.Success) loaded = true }
            )
        }

        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        0f to Color.Black.copy(alpha = 0.25f),
                        0.5f to Color.Black.copy(alpha = 0.55f),
                        1f to Color.Black.copy(alpha = 0.72f)
                    )
                )
        )

        Box(
            modifier = Modifier
                .fillMaxSize()
                .graphicsLayer { alpha = 0.35f }
                .background(theme.backgroundBrush)
        )
    }
}
