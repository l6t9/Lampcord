package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.UserActivity
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.ui.components.UsernameView
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ProfileHeader(
    profile: UserProfile,
    theme: ProfileTheme,
    isExpanded: Boolean,
    onExpand: (() -> Unit)? = null,
    userStore: UserStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    val currentUser by userStore.currentUser.collectAsState()

    val avatarUrl = profile.guild_member?.avatar?.let {
        "https://cdn.discordapp.com/guilds/${profile.guild_id}/users/${user.id}/avatars/$it.png?size=160"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160"
    }

    val presences by presenceStore.presences.collectAsState()
    val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]

    Row(verticalAlignment = Alignment.Bottom) {
        Box(
            modifier = Modifier
                .offset(y = if (isExpanded) (-60).dp else (-45).dp)
                .size(if (isExpanded) 120.dp else 94.dp)
                .background(theme.cutoutColor, CircleShape)
                .padding(if (isExpanded) 8.dp else 6.dp)
        ) {
            val status = remember(presence, user.id, currentUser?.id, settingsStore.userSettings?.status) {
                if (user.id == currentUser?.id) {
                    settingsStore.userSettings?.status ?: "online"
                } else {
                    presenceStore.getUserStatus(user.id, presence, currentUser?.id, settingsStore.userSettings?.status)
                }
            }
            AvatarWithDecoration(
                avatarUrl = avatarUrl,
                decorationData = profile.guild_member?.avatar_decoration_data ?: user.avatar_decoration_data,
                size = if (isExpanded) 104.dp else 82.dp,
                status = status,
                modifier = Modifier.clickable(
                    enabled = !isExpanded,
                    indication = null,
                    interactionSource = remember { MutableInteractionSource() }
                ) { onExpand?.invoke() }
            )
        }

        val activities = profile.activities.ifEmpty { presence?.activities ?: emptyList() }
        val customStatus = activities.find { it.type == 4 }
        val otherActivity = activities.find { it.type != 4 }

        if (customStatus != null) {
            UserActivity(
                activity = customStatus,
                compact = true,
                modifier = Modifier
                    .offset(y = if (isExpanded) (-50).dp else (-35).dp)
                    .padding(start = 12.dp, bottom = 8.dp)
            )
        } else if (otherActivity != null) {
            UserActivity(
                activity = otherActivity,
                compact = true,
                modifier = Modifier
                    .offset(y = if (isExpanded) (-50).dp else (-35).dp)
                    .padding(start = 12.dp, bottom = 8.dp)
            )
        }
    }

    Column(modifier = Modifier.offset(y = if (isExpanded) (-50).dp else (-35).dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            UsernameView(
                name = profile.guild_member?.nick ?: user.global_name ?: user.username ?: "Unknown User",
                style = profile.guild_member?.display_name_styles ?: user.display_name_styles,
                baseStyle = if (isExpanded) MaterialTheme.typography.headlineSmall else MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = theme.contentColor,
                marquee = true
            )
            user.primary_guild?.let {
                Spacer(Modifier.width(4.dp))
                ClanTagView(it)
            }
            UserTagView(user, modifier = Modifier.padding(start = 4.dp))
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(user.username ?: "", style = MaterialTheme.typography.bodyMedium, color = theme.contentColor.copy(alpha = 0.9f))
            val pronouns = guildMeta?.pronouns.takeIf { !it.isNullOrBlank() } ?: userMeta?.pronouns.takeIf { !it.isNullOrBlank() } ?: user.pronouns
            if (!pronouns.isNullOrBlank()) {
                Text(
                    " • $pronouns",
                    style = MaterialTheme.typography.bodyMedium,
                    color = theme.contentColor.copy(alpha = 0.7f),
                    modifier = Modifier.padding(start = 4.dp).basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp),
                    maxLines = 1
                )
            }
        }
        Spacer(Modifier.height(8.dp))
        UserBadges(userId = user.id, badges = profile.badges + profile.guild_badges)
        
        if (user.id == currentUser?.id) {
            Spacer(Modifier.height(8.dp))
            val isServerProfile = profile.guild_member != null && profile.guild_id != null
            
            if (isServerProfile) {
                ButtonGroup(
                    overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                    expandedRatio = 1f,
                    horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
                    modifier = Modifier.fillMaxWidth().height(32.dp),
                ) {
                    customItem(
                        buttonGroupContent = {
                            Button(
                                onClick = { /* TODO: Edit User Profile */ },
                                shapes = ButtonDefaults.shapes(
                                    shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                    pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape,
                                ),
                                modifier = Modifier.weight(1f).fillMaxHeight(),
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = theme.buttonColor,
                                    contentColor = theme.buttonTextColor
                                ),
                                contentPadding = PaddingValues(horizontal = 12.dp)
                            ) {
                                Icon(Icons.Filled.Edit, null, modifier = Modifier.size(16.dp))
                                Spacer(Modifier.width(8.dp))
                                Text("User Profile", fontSize = 12.sp, fontWeight = FontWeight.Bold, maxLines = 1)
                            }
                        },
                        menuContent = { menuState ->
                            DropdownMenuItem(
                                leadingIcon = { Icon(Icons.Filled.Edit, null) },
                                text = { Text("Edit User Profile") },
                                onClick = {
                                    /* TODO */
                                    menuState.dismiss()
                                }
                            )
                        }
                    )
                    customItem(
                        buttonGroupContent = {
                            Button(
                                onClick = { /* TODO: Edit Server Profile */ },
                                shapes = ButtonDefaults.shapes(
                                    shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                    pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape,
                                ),
                                modifier = Modifier.weight(1f).fillMaxHeight(),
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = theme.buttonColor,
                                    contentColor = theme.buttonTextColor
                                ),
                                contentPadding = PaddingValues(horizontal = 12.dp)
                            ) {
                                Icon(Icons.Filled.Edit, null, modifier = Modifier.size(16.dp))
                                Spacer(Modifier.width(8.dp))
                                Text("Server Profile", fontSize = 12.sp, fontWeight = FontWeight.Bold, maxLines = 1)
                            }
                        },
                        menuContent = { menuState ->
                            DropdownMenuItem(
                                leadingIcon = { Icon(Icons.Filled.Edit, null) },
                                text = { Text("Edit Server Profile") },
                                onClick = {
                                    /* TODO */
                                    menuState.dismiss()
                                }
                            )
                        }
                    )
                }
            } else {
                Button(
                    onClick = { /* TODO: Edit Profile */ },
                    modifier = Modifier.fillMaxWidth().height(32.dp),
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
            }
        }
        Spacer(Modifier.height(8.dp))
    }
}
