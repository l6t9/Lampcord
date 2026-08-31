package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.UserActivity
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.ui.components.UsernameView
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ProfileHeader(
    profile: UserProfile,
    theme: ProfileTheme,
    isExpanded: Boolean,
    onExpand: (() -> Unit)? = null,
    onDismiss: (() -> Unit)? = null,
    onEditAvatar: (() -> Unit)? = null,
    customProfileOverride: me.lampu.lampcord.shared.model.CustomProfile? = null,
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    relationshipStore: RelationshipStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    val currentUser by userStore.currentUser.collectAsState()
    val relationships by relationshipStore.relationships.collectAsState()

    val relationship = remember(relationships, user.id) {
        relationships.find { (it.id ?: it.user?.id ?: it.user_id) == user.id }
    }
    val isFriend = relationship?.type == 1

    val clientProfileStore: ClientProfileStore = koinInject()
    val customProfiles by clientProfileStore.customProfiles.collectAsState()
    val dbProfile = remember(user.id, settingsStore.userPfp, customProfiles) { 
        clientProfileStore.getCustomProfile(user.id)?.let {
            if (!settingsStore.userPfp) it.copy(avatar = null) else it
        }
    }

    val customProfile = customProfileOverride ?: remember(dbProfile) { dbProfile }

    val avatarUrl = customProfile?.avatar ?: profile.guild_member?.avatar?.let {
        "https://cdn.discordapp.com/guilds/${profile.guild_id}/users/${user.id}/avatars/$it.png?size=160"
    } ?: CdnUrls.getUserAvatarUrl(user.id, user.avatar, 160)

    val presences by presenceStore.presences.collectAsState()
    val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]

    val activities = remember(profile.activities, presence, presences[user.id], user.id, currentUser?.id) {
        val reactivePresence = if (user.id == currentUser?.id) presences[user.id] ?: presence else presence
        profile.activities.ifEmpty { reactivePresence?.activities ?: emptyList() }
    }
    val customStatus = activities.find { it.type == 4 }
    val otherActivity = activities.find { it.type != 4 }
    val displayActivity = customStatus ?: otherActivity

    val profileTextColor = MaterialTheme.colorScheme.onSurface
    val profileSecondaryTextColor = MaterialTheme.colorScheme.onSurfaceVariant

    Row(
        verticalAlignment = Alignment.Top, // Align to top to ensure downward expansion
        modifier = Modifier.fillMaxWidth().padding(end = 16.dp).zIndex(1f)
    ) {
        Box(
            modifier = Modifier
                .offset(y = (-45).dp)
                .size(94.dp)
                .background(theme.cutoutColor, CircleShape)
                .padding(6.dp)
        ) {
            val status = remember(presence, user.id, currentUser?.id, settingsStore.userSettings?.status) {
                if (user.id == currentUser?.id) {
                    settingsStore.userSettings?.status ?: "online"
                } else {
                    presenceStore.getUserStatus(user.id, presence, currentUser?.id, settingsStore.userSettings?.status)
                }
            }
            val avatarInteractionSource = remember { MutableInteractionSource() }
            val isAvatarHovered by avatarInteractionSource.collectIsHoveredAsState()

            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .hoverable(avatarInteractionSource)
                    .then(if (onEditAvatar != null) Modifier.clickable { onEditAvatar() } else Modifier)
            ) {
                AvatarWithDecoration(
                    avatarUrl = avatarUrl,
                    decorationData = profile.guild_member?.avatar_decoration_data ?: user.avatar_decoration_data,
                    size = 82.dp,
                    status = status,
                    modifier = Modifier.then(
                        if (onEditAvatar == null) {
                            Modifier.clickable(
                                enabled = !isExpanded,
                                indication = null,
                                interactionSource = remember { MutableInteractionSource() }
                            ) { onExpand?.invoke() }
                        } else Modifier
                    )
                )

                if (onEditAvatar != null && isAvatarHovered) {
                    Surface(
                        modifier = Modifier.align(Alignment.Center),
                        color = Color.Black.copy(alpha = 0.4f),
                        shape = CircleShape
                    ) {
                        Icon(
                            imageVector = Icons.Filled.Edit,
                            contentDescription = "Edit Avatar",
                            modifier = Modifier.padding(8.dp).size(20.dp),
                            tint = Color.White
                        )
                    }
                }
            }
        }

        if (displayActivity != null) {
            Box(
                modifier = Modifier
                    .weight(1f, fill = false)
                    .zIndex(10f)
                    .offset(
                        x = (-8).dp,
                        y = (-42).dp
                    )
                    // Use layout to report a fixed small height so expansion doesn't push content below
                    .layout { measurable, constraints ->
                        val placeable = measurable.measure(constraints)
                        val fixedHeight = 24.dp.roundToPx()
                        layout(placeable.width, fixedHeight) {
                            placeable.placeRelative(0, 0)
                        }
                    }
            ) {
                // Smallest dot - lowered and solid
                Surface(
                    modifier = Modifier
                        .offset(x = 10.dp, y = 14.dp)
                        .size(10.dp),
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.surfaceContainerLow,
                    tonalElevation = 2.dp,
                    shadowElevation = 1.dp
                ) {}
                
                // Medium dot - bigger and deeply submerged
                Surface(
                    modifier = Modifier
                        .offset(x = 22.dp, y = 18.dp)
                        .size(24.dp),
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.surfaceContainerLow,
                    tonalElevation = 2.dp,
                    shadowElevation = 1.dp
                ) {}

                Surface(
                    modifier = Modifier
                        .offset(x = 32.dp, y = 28.dp)
                        .widthIn(max = 280.dp),
                    color = MaterialTheme.colorScheme.surfaceContainerLow,
                    shape = RoundedCornerShape(12.dp),
                    tonalElevation = 4.dp,
                    shadowElevation = 2.dp
                ) {
                    UserActivity(
                        activity = displayActivity,
                        compact = true,
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp)
                    )
                }
            }
        }
    }

    Column(modifier = Modifier.offset(y = (-35).dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            UsernameView(
                name = profile.guild_member?.nick ?: user.global_name ?: user.username ?: "Unknown User",
                style = profile.guild_member?.display_name_styles ?: user.display_name_styles,
                baseStyle = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = profileTextColor,
                marquee = !Settings.shared.reduceMotion
            )
            user.primary_guild?.let {
                ClanTagView(it)
            }
            UserTagView(user)
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(user.username ?: "", style = MaterialTheme.typography.bodyMedium, color = profileTextColor.copy(alpha = 0.9f))
            val pronouns = guildMeta?.pronouns.takeIf { !it.isNullOrBlank() } ?: userMeta?.pronouns.takeIf { !it.isNullOrBlank() } ?: user.pronouns
            if (!pronouns.isNullOrBlank()) {
                Text(
                    " • $pronouns",
                    style = MaterialTheme.typography.bodyMedium,
                    color = profileSecondaryTextColor,
                    modifier = Modifier.padding(start = 4.dp).then(
                        if (Settings.shared.reduceMotion) Modifier else Modifier.basicMarquee(
                            iterations = if (getPlatformName() == "windows") 1 else Int.MAX_VALUE,
                            initialDelayMillis = 3000,
                            velocity = 30.dp
                        )
                    ),
                    maxLines = 1
                )
            }
        }
        Spacer(Modifier.height(8.dp))
        UserBadges(userId = user.id, badges = profile.badges + profile.guild_badges)
        
        if (user.id == currentUser?.id && onEditAvatar == null) {
            Spacer(Modifier.height(8.dp))
            Button(
                onClick = {
                    navigationStore.navigateToSettings("PROFILES")
                    onDismiss?.invoke()
                },
                modifier = Modifier.fillMaxWidth().height(40.dp),
                shape = CircleShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = theme.buttonColor,
                    contentColor = theme.buttonTextColor
                ),
                contentPadding = PaddingValues(horizontal = 12.dp)
            ) {
                Icon(Icons.Filled.Edit, null, modifier = Modifier.size(16.dp))
                Spacer(Modifier.width(8.dp))
                Text("Edit Profile", style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Medium)
            }
        } else if (user.id != currentUser?.id) {
            Spacer(Modifier.height(12.dp))
            Row(
                modifier = Modifier.fillMaxWidth().height(40.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Button(
                    onClick = { 
                        navigationStore.openDm(user.id)
                        onDismiss?.invoke()
                    },
                    modifier = Modifier.weight(1f).fillMaxHeight(),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = theme.buttonColor,
                        contentColor = theme.buttonTextColor
                    ),
                    shape = CircleShape,
                    contentPadding = PaddingValues(horizontal = 12.dp)
                ) {
                    Icon(
                        imageVector = if (isFriend) Icons.Filled.Chat else Icons.Filled.PersonAdd,
                        contentDescription = if (isFriend) "Message" else "Add Friend",
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = if (isFriend) "Message" else "Add Friend",
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Medium
                    )
                }
            }
        }
    }
}
