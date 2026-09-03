package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.ContextMenu
import me.lampu.lampcord.shared.ui.components.ContextMenuItem
import me.lampu.lampcord.shared.ui.components.UserActivity
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.ui.components.UsernameView
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject

@Composable
fun MemberItem(
    member: Member,
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    messageStore: MessageStore = koinInject(),
    presenceStore: PresenceStore = koinInject()
) {
    val user = remember(member, userStore) {
        member.user ?: member.userId()?.let { userStore.getUser(it) }
    }
    
    val displayUser = user ?: member.user ?: me.lampu.lampcord.shared.model.User(id = member.userId() ?: return)
    
    val guildId = navigationStore.selectedGuild?.id
    val avatarUrl = member.avatar?.let {
        "https://cdn.discordapp.com/guilds/$guildId/users/${displayUser.id}/avatars/$it.png"
    } ?: CdnUrls.getUserAvatarUrl(displayUser.id, displayUser.avatar, 64)

    val roleData = remember(member.roles, navigationStore.selectedGuild) {
        val guild = navigationStore.selectedGuild ?: return@remember null
        val colorRole = member.getRoleColorRole(guild)
        
        if (colorRole != null) {
            val primaryInt = colorRole.colors?.primary_color ?: colorRole.color
            val gradient = if (colorRole.colors?.secondary_color != null) {
                listOfNotNull(
                    Color(primaryInt or 0xFF000000.toInt()),
                    Color(colorRole.colors.secondary_color or 0xFF000000.toInt()),
                    colorRole.colors.tertiary_color?.let { Color(it or 0xFF000000.toInt()) }
                )
            } else null
            val color = if (primaryInt != 0) Color(primaryInt or 0xFF000000.toInt()) else Color.Unspecified
            color to gradient
        } else null
    }

    val roleColor = roleData?.first ?: Color.Unspecified
    val roleGradient = roleData?.second

    val relationshipStore = koinInject<RelationshipStore>()
    val relationships by relationshipStore.relationships.collectAsState()
    val currentUserId = userStore.currentUser.collectAsState().value?.id
    val relationshipType = remember(relationships, displayUser.id) {
        relationships.find { (it.id ?: it.user?.id ?: it.user_id) == displayUser.id }?.type
    }

    val errorColor = MaterialTheme.colorScheme.error
    val contextMenuItems = remember(displayUser, settingsStore.userSettings, relationshipType, currentUserId, errorColor) {
        val isMe = displayUser.id == currentUserId
        val items = mutableListOf<ContextMenuItem>()
        items.add(ContextMenuItem("Profile", Icons.Filled.AccountCircle, onClick = { profileStore.showProfile(displayUser.id, guildId) }, group = "Primary"))
        items.add(ContextMenuItem("Mention", Icons.Rounded.AlternateEmail, onClick = {
            val channelId = navigationStore.selectedChannel?.id ?: return@ContextMenuItem
            val current = messageStore.draftMessages[channelId] ?: ""
            messageStore.draftMessages[channelId] = "$current <@${displayUser.id}> "
        }, group = "Primary"))
        items.add(ContextMenuItem("Message", Icons.Filled.Chat, onClick = { navigationStore.openDm(displayUser.id) }, group = "Primary"))

        if (!isMe) {
            when (relationshipType) {
                1 -> items.add(ContextMenuItem("Remove Friend", Icons.Filled.PersonRemove, onClick = {
                    relationshipStore.removeFriend(displayUser.id)
                }, group = "Social"))
                2 -> items.add(ContextMenuItem("Unblock", Icons.Filled.Block, onClick = {
                    relationshipStore.unblockUser(displayUser.id)
                }, group = "Social"))
                3 -> items.add(ContextMenuItem("Accept Friend Request", Icons.Filled.PersonAdd, onClick = {
                    relationshipStore.addFriend(displayUser.id)
                }, group = "Social"))
                else -> items.add(ContextMenuItem("Add Friend", Icons.Filled.PersonAdd, onClick = {
                    relationshipStore.addFriend(displayUser.id)
                }, group = "Social"))
            }
            items.add(ContextMenuItem("Block", Icons.Filled.Block, onClick = {
                relationshipStore.blockUser(displayUser.id)
            }, color = errorColor, group = "Destructive"))
        }
        if (settingsStore.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy User ID", Icons.Filled.Dns, onClick = { setClipboardText(displayUser.id) }, group = "Developer"))
        }
        items
    }

    var itemPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
    var isHovered by remember { mutableStateOf(false) }

    val presences by presenceStore.presences.collectAsState()
    val presence = remember(member.presence, presences[displayUser.id]) {
        member.presence ?: presences[displayUser.id]
    }
    val isStreaming = presence?.activities?.any { it.type == 1 } == true
    val isListening = presence?.activities?.any { it.type == 2 } == true
    val isStatusVisible = presenceStore.isStatusVisible(displayUser, presence)
    
    val isOffline = !isStatusVisible && !isListening

    val nameplate = member.collectibles?.nameplate ?: displayUser.collectibles?.nameplate

    ContextMenu(
        items = contextMenuItems,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp)
            .onGloballyPositioned { itemPosition = it.positionInRoot() }
            .pointerInput(Unit) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent()
                        when (event.type) {
                            PointerEventType.Enter -> isHovered = true
                            PointerEventType.Exit -> isHovered = false
                        }
                    }
                }
            }
            .graphicsLayer {
                alpha = if (isOffline && !isHovered) 0.4f else 1f
            },
        shape = RoundedCornerShape(8.dp)
    ) {
        Surface(
            modifier = Modifier.fillMaxWidth().height(44.dp),
            onClick = { profileStore.showProfile(displayUser.id, guildId, position = itemPosition) },
            color = Color.Transparent,
            shape = RoundedCornerShape(8.dp)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                if (nameplate != null) {
                    val decoUrl = "https://cdn.discordapp.com/assets/collectibles/${nameplate.asset}img.png?passthrough=true"
                    AsyncImage(
                        model = decoUrl,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        alpha = 0.4f
                    )
                }

                Row(
                    modifier = Modifier.padding(horizontal = 8.dp).fillMaxSize(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(modifier = Modifier.size(32.dp)) {
                        val currentUser by userStore.currentUser.collectAsState()
                        AvatarWithDecoration(
                            avatarUrl = avatarUrl,
                            decorationData = member.avatar_decoration_data ?: displayUser.avatar_decoration_data,
                            size = 32.dp,
                            status = presenceStore.getUserStatus(displayUser.id, currentUser?.id, settingsStore.userSettings?.status)
                        )
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    Column {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            UsernameView(
                                name = member.nick ?: displayUser.global_name ?: displayUser.username ?: "Unknown User",
                                style = member.display_name_styles ?: displayUser.display_name_styles,
                                baseStyle = MaterialTheme.typography.bodyMedium,
                                color = roleColor,
                                roleGradient = roleGradient,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                marquee = true,
                                ignoreEffects = false,
                                ignoreColors = true,
                                modifier = Modifier.weight(1f, fill = false)
                            )
                            displayUser.primary_guild?.let {
                                ClanTagView(it)
                            }
                            UserTagView(displayUser)
                            
                            val guild = navigationStore.selectedGuild
                            if (displayUser.id == guild?.owner_id) {
                                Spacer(modifier = Modifier.width(4.dp))
                                Icon(
                                    imageVector = Icons.Filled.Crown,
                                    contentDescription = "Owner",
                                    modifier = Modifier.size(14.dp),
                                    tint = Color(0xFFF9A825)
                                )
                            }
                        }
                        
                        val activities = (member.presence ?: presence)?.activities ?: emptyList()
                        val customStatus = activities.find { it.type == 4 }
                        val otherActivity = activities.find { it.type != 4 }
                        
                        if (customStatus != null) {
                            UserActivity(customStatus, compact = true, modifier = Modifier.alpha(0.7f))
                        } else if (otherActivity != null) {
                            UserActivity(otherActivity, compact = true, modifier = Modifier.alpha(0.7f))
                        }
                    }
                }
            }
        }
    }
}
