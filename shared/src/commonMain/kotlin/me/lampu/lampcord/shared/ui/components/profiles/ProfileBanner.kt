package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.model.EmbedImage
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.components.ImageLoadState
import me.lampu.lampcord.shared.ui.kit.combinedClickableCursor
import me.lampu.lampcord.shared.ui.kit.handCursor

internal fun profileBannerHeight(
    width: Dp,
    isExpanded: Boolean,
    heightOverride: Dp?,
    heightRatio: Float,
): Dp = when {
    heightRatio > 0f -> (width * heightRatio).coerceIn(80.dp, 420.dp)
    else -> heightOverride ?: if (isExpanded) 160.dp else 105.dp
}

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable
fun ProfileBanner(
    profile: UserProfile,
    theme: ProfileTheme,
    isExpanded: Boolean,
    bannerHeightOverride: Dp? = null,
    topShape: Shape? = null,
    bannerHeightRatio: Float = 0f,
    contentScale: ContentScale = ContentScale.Crop,
    onDismiss: (() -> Unit)? = null,
    onEdit: (() -> Unit)? = null,
    customProfileOverride: me.lampu.lampcord.shared.model.CustomProfile? = null,
    relationshipStore: RelationshipStore = koinInject(),
    userStore: UserStore = koinInject(),
    clientProfileStore: ClientProfileStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    val guildId = profile.guild_id
    val currentUser by userStore.currentUser.collectAsState()
    val relationships by relationshipStore.relationships.collectAsState()
    val customProfiles by clientProfileStore.customProfiles.collectAsState()
    val dbProfile = remember(user.id, settingsStore.userBg, customProfiles) { 
        clientProfileStore.getCustomProfile(user.id)?.let {
            if (!settingsStore.userBg) it.copy(banner = null) else it
        }
    }
    val customProfile = customProfileOverride ?: dbProfile

    val relationship = remember(relationships, user.id) {
        relationships.find { (it.id ?: it.user?.id ?: it.user_id) == user.id }
    }
    val isFriend = relationship?.type == 1
    val isBlocked = relationship?.type == 2

    val platform = remember { me.lampu.lampcord.shared.utils.getPlatformName() }
    BoxWithConstraints {
    val bannerHeight = remember(maxWidth, bannerHeightRatio, bannerHeightOverride, isExpanded) {
        profileBannerHeight(maxWidth, isExpanded, bannerHeightOverride, bannerHeightRatio)
    }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(bannerHeight)
            .then(if (topShape != null) Modifier.clip(topShape) else Modifier)
    ) {
        val bannerUrl = if (customProfile?.banner != null) {
            customProfile.banner
        } else if (guildMeta?.banner != null && guildId != null) {
            "https://cdn.discordapp.com/guilds/$guildId/users/${user.id}/banners/${guildMeta.banner}.png?size=${if (isExpanded) 1024 else 600}"
        } else (userMeta?.banner ?: user.banner)?.let {
            "https://cdn.discordapp.com/banners/${user.id}/$it.png?size=${if (isExpanded) 1024 else 600}"
        }

        val interactionSource = remember { MutableInteractionSource() }
        val isHovered by interactionSource.collectIsHoveredAsState()

        Box(
            modifier = Modifier
                .fillMaxSize()
                .hoverable(interactionSource)
                .combinedClickableCursor(
                    onClick = {
                        if (onEdit != null) {
                            onEdit()
                        } else if (bannerUrl != null) {
                            navigationStore.openAttachmentViewer(listOf(EmbedImage(url = bannerUrl, proxy_url = bannerUrl)))
                        }
                    },
                    onLongClick = {
                        setClipboardText(user.id)
                        showToast("Copied User ID: ${user.id}")
                    }
                )
        ) {
            var isImageLoaded by remember { mutableStateOf(false) }
            
            if (bannerUrl != null) {
                AsyncImage(
                    model = bannerUrl,
                    contentDescription = "Profile Banner",
                    contentScale = contentScale,
                    modifier = Modifier.fillMaxSize(),
                    filterQuality = FilterQuality.Medium,
                    onState = { state ->
                        if (state is ImageLoadState.Success) {
                            isImageLoaded = true
                        }
                    }
                )
            }

            // Fallback/Loading background color that always exists
            if (!isImageLoaded) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(theme.primaryAccent.copy(alpha = if (bannerUrl == null) 0.5f else 0.2f))
                )
            }
            
            if (onEdit != null && isHovered) {
                Surface(
                    modifier = Modifier.align(Alignment.Center),
                    color = Color.Black.copy(alpha = 0.4f),
                    shape = CircleShape
                ) {
                    Icon(
                        imageVector = Icons.Filled.Edit,
                        contentDescription = "Edit Banner",
                        modifier = Modifier.padding(8.dp).size(24.dp),
                        tint = Color.White
                    )
                }
            }
        }

        var menuExpanded by remember { mutableStateOf(false) }
        var showNicknameDialog by remember { mutableStateOf(false) }

        if (showNicknameDialog) {
            var nickname by remember { mutableStateOf(relationship?.nickname ?: "") }
            AlertDialog(
                onDismissRequest = { showNicknameDialog = false },
                title = { Text("Edit Friend Nickname") },
                text = {
                    OutlinedTextField(
                        value = nickname,
                        onValueChange = { nickname = it },
                        label = { Text("Nickname") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true
                    )
                },
                confirmButton = {
                    TextButton(onClick = {
                        relationshipStore.updateNickname(user.id, nickname.takeIf { it.isNotBlank() })
                        showNicknameDialog = false
                    }) {
                        Text("Save")
                    }
                },
                dismissButton = {
                    TextButton(onClick = { showNicknameDialog = false }) {
                        Text("Cancel")
                    }
                }
            )
        }

        Box(
            modifier = Modifier
                .align(Alignment.TopEnd)
                .padding(
                    top = if (platform == "android" && isExpanded) 16.dp else 8.dp,
                    end = if (platform == "android" && isExpanded) 16.dp else 8.dp
                )
        ) {
            Surface(
                onClick = { menuExpanded = true },
                modifier = Modifier.size(32.dp),
                shape = CircleShape,
                color = Color.Black.copy(alpha = 0.4f),
                contentColor = Color.White
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Filled.MoreHoriz,
                        contentDescription = "More options",
                        modifier = Modifier.size(20.dp)
                    )
                }
            }

            DropdownMenu(
                expanded = menuExpanded,
                onDismissRequest = { menuExpanded = false }
            ) {
                if (user.id != currentUser?.id) {
                    if (isFriend) {
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Remove Friend") },
                            onClick = {
                                relationshipStore.removeFriend(user.id)
                                menuExpanded = false
                            },
                            leadingIcon = { Icon(Icons.Filled.PersonRemove, null) }
                        )
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Edit Friend Nickname") },
                            onClick = {
                                showNicknameDialog = true
                                menuExpanded = false
                            },
                            leadingIcon = { Icon(Icons.Filled.Edit, null) }
                        )
                    } else if (!isBlocked) {
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Add Friend") },
                            onClick = {
                                relationshipStore.addFriend(user.id)
                                menuExpanded = false
                            },
                            leadingIcon = { Icon(Icons.Filled.PersonAdd, null) }
                        )
                    }

                    HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))

                    DropdownMenuItem(
                        modifier = Modifier.handCursor(),
                        text = { Text("Block", color = MaterialTheme.colorScheme.error) },
                        onClick = {
                            relationshipStore.blockUser(user.id)
                            menuExpanded = false
                        },
                        leadingIcon = { Icon(Icons.Filled.Block, null, tint = MaterialTheme.colorScheme.error) }
                    )
                }

                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Text("Copy User ID") },
                    onClick = {
                        me.lampu.lampcord.shared.utils.setClipboardText(user.id)
                        menuExpanded = false
                    },
                    leadingIcon = { Icon(Icons.Filled.ContentCopy, null) }
                )
            }
        }
    }
    }
}
