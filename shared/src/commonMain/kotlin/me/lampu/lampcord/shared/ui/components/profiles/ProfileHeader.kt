package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.TabRowDefaults.tabIndicatorOffset
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AvatarWithDecoration
import me.lampu.lampcord.shared.ui.components.ClanTagView
import me.lampu.lampcord.shared.ui.components.UserActivity
import me.lampu.lampcord.shared.ui.components.UserTagView
import me.lampu.lampcord.shared.ui.components.UsernameView
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ProfileHeader(
    profile: UserProfile,
    theme: ProfileTheme,
    isExpanded: Boolean,
    selectedTab: Int,
    onTabSelected: (Int) -> Unit,
    onExpand: (() -> Unit)? = null,
    onDismiss: (() -> Unit)? = null,
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
    val isBlocked = relationship?.type == 2

    val avatarUrl = profile.guild_member?.avatar?.let {
        "https://cdn.discordapp.com/guilds/${profile.guild_id}/users/${user.id}/avatars/$it.png?size=160"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160"
    }

    val presences by presenceStore.presences.collectAsState()
    val presence = profile.guild_member?.presence ?: profile.presence ?: presences[user.id]

    val profileTextColor = MaterialTheme.colorScheme.onSurface
    val profileSecondaryTextColor = MaterialTheme.colorScheme.onSurfaceVariant

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

        val activities = remember(profile.activities, presence, presences[user.id], user.id, currentUser?.id) {
            val reactivePresence = if (user.id == currentUser?.id) presences[user.id] ?: presence else presence
            profile.activities.ifEmpty { reactivePresence?.activities ?: emptyList() }
        }
        val customStatus = activities.find { it.type == 4 }
        val otherActivity = activities.find { it.type != 4 }
        val displayActivity = customStatus ?: otherActivity

        if (displayActivity != null) {
            Surface(
                color = MaterialTheme.colorScheme.surfaceContainerLow.copy(alpha = 0.8f),
                shape = RoundedCornerShape(
                    topStart = 16.dp,
                    topEnd = 16.dp,
                    bottomEnd = 16.dp,
                    bottomStart = 4.dp
                ),
                tonalElevation = 4.dp,
                shadowElevation = 2.dp,
                modifier = Modifier
                    .offset(y = if (isExpanded) (-54).dp else (-39).dp)
                    .padding(start = 6.dp, bottom = 12.dp)
            ) {
                UserActivity(
                    activity = displayActivity,
                    compact = true,
                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp)
                )
            }
        }
    }

    Column(modifier = Modifier.offset(y = if (isExpanded) (-50).dp else (-35).dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            UsernameView(
                name = profile.guild_member?.nick ?: user.global_name ?: user.username ?: "Unknown User",
                style = profile.guild_member?.display_name_styles ?: user.display_name_styles,
                baseStyle = if (isExpanded) MaterialTheme.typography.headlineSmall else MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = profileTextColor,
                marquee = true
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
                    modifier = Modifier.padding(start = 4.dp).basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp),
                    maxLines = 1
                )
            }
        }
        Spacer(Modifier.height(8.dp))
        UserBadges(userId = user.id, badges = profile.badges + profile.guild_badges)
        
        Spacer(Modifier.height(12.dp))
        if (user.id == currentUser?.id) {
            val isServerProfile = profile.guild_member != null && profile.guild_id != null
            
            if (isServerProfile) {
                ButtonGroup(
                    overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                    expandedRatio = 1f,
                    horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
                    modifier = Modifier.fillMaxWidth().height(40.dp),
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
                                Text("User Profile", fontSize = 13.sp, fontWeight = FontWeight.Bold, maxLines = 1)
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
                                Text("Server Profile", fontSize = 13.sp, fontWeight = FontWeight.Bold, maxLines = 1)
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
            }
        } else {
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

                Box {
                    Button(
                        onClick = { menuExpanded = true },
                        modifier = Modifier.size(40.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = theme.cardColor,
                            contentColor = profileTextColor
                        ),
                        shape = CircleShape,
                        contentPadding = PaddingValues(0.dp)
                    ) {
                        Icon(Icons.Filled.MoreHoriz, null, modifier = Modifier.size(18.dp))
                    }

                    DropdownMenu(
                        expanded = menuExpanded,
                        onDismissRequest = { menuExpanded = false }
                    ) {
                        if (isFriend) {
                            DropdownMenuItem(
                                text = { Text("Remove Friend") },
                                onClick = {
                                    relationshipStore.removeFriend(user.id)
                                    menuExpanded = false
                                },
                                leadingIcon = { Icon(Icons.Filled.PersonRemove, null) }
                            )
                            DropdownMenuItem(
                                text = { Text("Edit Friend Nickname") },
                                onClick = {
                                    showNicknameDialog = true
                                    menuExpanded = false
                                },
                                leadingIcon = { Icon(Icons.Filled.Edit, null) }
                            )
                        } else if (!isBlocked) {
                            DropdownMenuItem(
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
                            text = { Text("Block", color = MaterialTheme.colorScheme.error) },
                            onClick = {
                                relationshipStore.blockUser(user.id)
                                menuExpanded = false
                            },
                            leadingIcon = { Icon(Icons.Filled.Block, null, tint = MaterialTheme.colorScheme.error) }
                        )
                        
                        DropdownMenuItem(
                            text = { Text("Ignore") },
                            onClick = {
                                // TODO: Ignore user
                                menuExpanded = false
                            },
                            leadingIcon = { Icon(Icons.Filled.VisibilityOff, null) }
                        )
                    }
                }
            }
        }
        
        Spacer(Modifier.height(16.dp))
        
        // Main / Board Tabs
        PrimaryTabRow(
            selectedTabIndex = selectedTab,
            containerColor = Color.Transparent,
            contentColor = profileTextColor,
            divider = {}
        ) {
            val tabs = listOf("Main", "Board")
            tabs.forEachIndexed { index, title ->
                Tab(
                    selected = selectedTab == index,
                    onClick = { onTabSelected(index) },
                    text = {
                        Text(
                            text = title,
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = if (selectedTab == index) FontWeight.Bold else FontWeight.Medium,
                            color = if (selectedTab == index) profileTextColor else profileSecondaryTextColor
                        )
                    }
                )
            }
        }
        Spacer(Modifier.height(8.dp))
    }
}
