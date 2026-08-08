package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText

@Composable
fun MemberItem(member: Member, chatState: ChatState) {
    val user = remember(member, chatState.userStore) {
        member.user ?: member.userId()?.let { chatState.userStore.getUser(it) }
    }
    
    if (user == null) return
    val avatarUrl = member.avatar?.let {
        "https://cdn.discordapp.com/guilds/${chatState.selectedGuild?.id}/users/${user.id}/avatars/$it.png"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
    }

    val roleColor = remember(member.roles, chatState.selectedGuild) {
        val guild = chatState.selectedGuild ?: return@remember Color.Unspecified
        val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
        val highestRole = memberRoles.maxByOrNull { it.position }
        if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.Unspecified
    }

    val contextMenuItems = remember(user, chatState.userSettings) {
        val items = mutableListOf(
            ContextMenuItem("Profile", Icons.Filled.AccountCircle) { chatState.showProfile(user.id) },
            ContextMenuItem("Mention", Icons.Outlined.AlternateEmail) {
                val channelId = chatState.selectedChannel?.id ?: return@ContextMenuItem
                val current = chatState.draftMessages[channelId] ?: ""
                chatState.draftMessages[channelId] = "$current <@${user.id}> "
            },
            ContextMenuItem("Message", Icons.Filled.Share) { /* TODO */ }
        )
        if (chatState.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy User ID", Icons.Filled.Dns) { setClipboardText(user.id) })
        }
        items
    }

    var itemPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
    var isHovered by remember { mutableStateOf(false) }

    val status = chatState.getUserStatus(user.id)
    val isListening = member.presence?.activities?.any { it.type == 2 } == true
    val isOffline = (status == "offline" || status == "invisible") && !isListening

    val nameplate = member.collectibles?.nameplate ?: user.collectibles?.nameplate

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
            onClick = { chatState.showProfile(user.id, itemPosition) },
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
                        AvatarWithDecoration(
                            avatarUrl = avatarUrl,
                            decorationData = member.avatar_decoration_data ?: user.avatar_decoration_data,
                            size = 32.dp,
                            status = chatState.getUserStatus(user.id)
                        )
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    Column {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            UsernameView(
                                name = member.nick ?: user.global_name ?: user.username ?: "Unknown User",
                                style = member.display_name_styles ?: user.display_name_styles,
                                baseStyle = MaterialTheme.typography.bodyMedium,
                                color = if (roleColor != Color.Unspecified) roleColor else MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                ignoreEffects = true,
                                ignoreColors = true
                            )
                            user.primary_guild?.let {
                                Spacer(Modifier.width(4.dp))
                                ClanTagView(it)
                            }
                            UserTagView(user, modifier = Modifier.padding(start = 4.dp))
                        }
                        
                        val activities = member.presence?.activities ?: emptyList()
                        val customStatus = activities.find { it.type == 4 }
                        val otherActivity = activities.find { it.type != 4 }
                        
                        if (customStatus != null) {
                            UserActivity(customStatus, compact = true)
                        } else if (otherActivity != null) {
                            UserActivity(otherActivity, compact = true)
                        }
                    }
                }
            }
        }
    }
}
