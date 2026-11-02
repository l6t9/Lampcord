package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.ProfileCollectibles
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ClientProfileStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.VerticalScrollbar
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private val CARD_WIDTH = 380.dp
private val AVATAR_SCALE = 1.45f
private const val BANNER_RATIO = 140f / 340f
private val CARD_CORNER = 16.dp
private val PANEL_CORNER = 28.dp
private val OUTER_CORNER = 28.dp
private val PANEL_MARGIN_H = 36.dp
private val PANEL_MARGIN_V = 28.dp

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
    val customProfile = resolved.customProfile

    val frameSku = remember(profile) {
        ProfileCollectibles.frameSku(profile.user_profile, profile.guild_member_profile)
    }
    val frameProduct = frameSku?.let { profileStore.getFrame(it) }
    val frameRailTop = profileBannerHeight(CARD_WIDTH, true, null, BANNER_RATIO)
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
                .background(Color.Black.copy(alpha = 0.58f))
                .combinedClickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = onDismiss,
                    onLongClick = {}
                )
        )

        Box(
            modifier = Modifier.align(Alignment.Center)
        ) {
            Box(
                modifier = Modifier
                    .padding(
                        start = PANEL_MARGIN_H,
                        top = PANEL_MARGIN_V,
                        end = PANEL_MARGIN_H,
                        bottom = PANEL_MARGIN_V
                    )
                    .widthIn(max = 1240.dp)
                    .fillMaxWidth(0.95f)
                    .heightIn(max = 880.dp)
                    .fillMaxHeight(0.9f)
            ) {
                Box(
                    modifier = Modifier
                        .matchParentSize()
                        .clip(RoundedCornerShape(OUTER_CORNER))
                ) {
                    MaterialProfileBackdrop()
                }

                Column(Modifier.fillMaxSize()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(64.dp)
                            .padding(start = 24.dp, end = 12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = "Profile",
                            style = MaterialTheme.typography.titleLarge,
                            fontWeight = FontWeight.SemiBold,
                            color = MaterialTheme.colorScheme.onSurface,
                            modifier = Modifier.weight(1f)
                        )
                        Surface(
                            onClick = onDismiss,
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.surfaceContainerHigh,
                            contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.size(40.dp)
                        ) {
                            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                Icon(
                                    imageVector = Icons.Filled.Close,
                                    contentDescription = "Close",
                                    modifier = Modifier.size(20.dp)
                                )
                            }
                        }
                    }

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f)
                            .padding(start = cardInset, end = cardInset, bottom = cardInset, top = 4.dp),
                        horizontalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .width(CARD_WIDTH)
                                .fillMaxHeight()
                        ) {
                        Surface(
                            modifier = Modifier.fillMaxSize(),
                            shape = RoundedCornerShape(CARD_CORNER),
                            color = Color.Transparent
                        ) {
                            Box(Modifier.fillMaxSize()) {
                                ProfileCard(
                                    profile = profile,
                                    modifier = Modifier.fillMaxSize(),
                                    isExpanded = true,
                                    fillAvailableHeight = true,
                                    showBorder = false,
                                    showBoardTab = false,
                                    showFrameArt = false,
                                    avatarScale = AVATAR_SCALE,
                                    topShape = RoundedCornerShape(CARD_CORNER),
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
                        }

                        frameProduct?.let { product ->
                            ProfileFrameOverlay(
                                product = product,
                                modifier = Modifier.matchParentSize(),
                                railTop = frameRailTop,
                                showBottom = false
                            )
                        }
                    }

                    Surface(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight(),
                        shape = RoundedCornerShape(PANEL_CORNER),
                        color = Color.Transparent
                    ) {
                        FullProfileDetails(
                            profile = profile,
                            topShape = RoundedCornerShape(PANEL_CORNER),
                            onOpenProfile = { userId -> profileStore.showProfile(userId) },
                            onOpenGuild = { guildId ->
                                guildStore.guilds.value.find { it.id == guildId }?.let {
                                    navigationStore.selectedGuild = it
                                }
                                profileStore.closeFullProfile()
                            },
                            modifier = Modifier.fillMaxSize()
                        )
                    }
                }
            }

        }
    }
}
}

@Composable
private fun MaterialProfileBackdrop() {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    listOf(
                        MaterialTheme.colorScheme.surfaceContainerLowest,
                        MaterialTheme.colorScheme.surfaceContainerLow
                    )
                )
            )
    )
}
