package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberBottomSheetState
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.rememberGuildMediaUrls
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserGuildSettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.DiscordBottomSheet
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.ui.components.settings.switchSettingsItem
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.chat.InviteDialog
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.handCursor
import me.lampu.lampcord.shared.ui.components.rememberDiscordSheetState

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ServerBottomSheet(
    guild: Guild,
    onDismiss: () -> Unit,
    sheetState: SheetState = rememberDiscordSheetState(),
    userGuildSettingsStore: UserGuildSettingsStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    userStore: UserStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
) {
    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()
    var showLeaveDialog by remember { mutableStateOf(false) }
    var showInviteDialog by remember { mutableStateOf(false) }
    val allChannels by guildStore.allGuildChannels.collectAsState()
    val inviteChannel = remember(guild.id, allChannels) {
        allChannels.values.find { it.guild_id == guild.id && it.type == 0 } ?: allChannels.values.find { it.guild_id == guild.id }
    }

    if (showLeaveDialog) {
        LeaveServerDialog(
            guildName = guild.name ?: "this server",
            onDismiss = { showLeaveDialog = false },
            onConfirm = {
                guildStore.leaveGuild(guild.id) { if (navigationStore.selectedGuild?.id == guild.id) navigationStore.selectHome() }
                showLeaveDialog = false
                onDismiss()
            }
        )
    }

    DiscordBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = MaterialTheme.colorScheme.surface
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(bottom = 32.dp)
        ) {
            Box(modifier = Modifier.fillMaxWidth()) {
                val guildMedia = rememberGuildMediaUrls(
                    guildId = guild.id,
                    guildIconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=160" },
                    guildBannerUrl = guild.banner?.let { "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600" }
                )
                val bannerUrl = guildMedia.banner
                if (bannerUrl != null) {
                    AsyncImage(
                        model = bannerUrl,
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier
                            .fillMaxWidth()
                            .aspectRatio(16f / 9f)
                    )
                } else {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(80.dp)
                            .background(MaterialTheme.colorScheme.primaryContainer)
                    )
                }
                
                Surface(
                    modifier = Modifier
                        .padding(start = 16.dp)
                        .align(Alignment.BottomStart)
                        .offset(y = 40.dp)
                        .size(80.dp),
                    shape = RoundedCornerShape(24.dp),
                    color = MaterialTheme.colorScheme.surfaceContainer,
                    border = androidx.compose.foundation.BorderStroke(4.dp, MaterialTheme.colorScheme.surfaceContainer)
                ) {
                    val iconUrl = guildMedia.icon
                    if (iconUrl != null) {
                        AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                    } else {
                        Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.secondaryContainer), contentAlignment = Alignment.Center) {
                            Text(guild.name?.take(1) ?: "?", style = MaterialTheme.typography.headlineMedium)
                        }
                    }
                }
            }

            if (showInviteDialog && inviteChannel != null) {
                InviteDialog(channel = inviteChannel, onDismiss = { showInviteDialog = false })
            }

            Spacer(Modifier.height(48.dp))

            Column(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(horizontal = 16.dp)) {
                    Text(
                        text = guild.name ?: "Server",
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold
                    )

                    if (guild.description?.isNotBlank() == true) {
                        Text(
                            text = guild.description,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(top = 4.dp)
                        )
                    }

                    Row(
                        modifier = Modifier.padding(top = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        // Online count if available, otherwise just member count
                        val onlineCount = remember(guild, memberListStore.getOnlineCount(guild.id)) {
                            guild.approximate_presence_count ?: memberListStore.getOnlineCount(guild.id)
                        }
                        if (onlineCount > 0) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Box(modifier = Modifier.size(8.dp).background(Color(0xFF23A559), CircleShape))
                                Spacer(Modifier.width(4.dp))
                                Text(
                                    "$onlineCount Online",
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Box(modifier = Modifier.size(8.dp).background(Color(0xFFB5BAC1), CircleShape))
                            Spacer(Modifier.width(4.dp))
                            val memberCount = guild.approximate_member_count ?: guild.member_count ?: memberListStore.getMemberCount(guild.id)
                            Text(
                                "${memberCount.takeIf { it > 0 } ?: 0} Members",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }

                    Spacer(Modifier.height(24.dp))

                    val member = remember(guild.id, currentUser) {
                        currentUser?.id?.let { userStore.getMember(guild.id, it) }
                    }
                    val canManageGuild = remember(guild, member) {
                        if (member == null) false
                        else PermissionHelper.hasPermission(member, guild, null, Permission.MANAGE_GUILD, currentUser?.id)
                    }

                    ButtonGroup(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(64.dp),
                        overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                        horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
                    ) {
                        val actions = mutableListOf(
                            Triple(Icons.Rounded.RocketLaunch, "${guild.premium_subscription_count ?: 0} Boosts") { /* TODO */ },
                            Triple(Icons.Rounded.Notifications, "Notifications") { /* TODO */ },
                            Triple(Icons.Filled.PersonAdd, "Invite") {
                                if (inviteChannel != null) showInviteDialog = true
                            }
                        )
                        
                        if (canManageGuild) {
                            actions.add(Triple(Icons.Rounded.Settings, "Settings") {
                                navigationStore.isServerSettingsVisible = true
                                onDismiss()
                            })
                        }

                        actions.forEachIndexed { index, (icon, label, onClick) ->
                            customItem(
                                buttonGroupContent = {
                                    val shapes = when {
                                        actions.size == 1 -> ButtonDefaults.shapes()
                                        index == 0 -> ButtonDefaults.shapes(
                                            shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                            pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape
                                        )
                                        index == actions.lastIndex -> ButtonDefaults.shapes(
                                            shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                            pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape
                                        )
                                        else -> ButtonDefaults.shapes(
                                            shape = MaterialTheme.shapes.small,
                                            pressedShape = ButtonGroupDefaults.connectedMiddleButtonPressShape
                                        )
                                    }

                                    FilledTonalButton(
                                        onClick = onClick,
                                        shapes = shapes,
                                        modifier = Modifier.weight(1f).fillMaxHeight(),
                                        contentPadding = PaddingValues(0.dp)
                                    ) {
                                        Column(
                                            horizontalAlignment = Alignment.CenterHorizontally,
                                            verticalArrangement = Arrangement.Center
                                        ) {
                                            Icon(
                                                imageVector = icon,
                                                contentDescription = null,
                                                modifier = Modifier.size(20.dp)
                                            )
                                            Spacer(Modifier.height(4.dp))
                                            Text(
                                                text = label,
                                                style = MaterialTheme.typography.labelSmall,
                                                textAlign = TextAlign.Center,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                                fontSize = 10.sp
                                            )
                                        }
                                    }
                                },
                                menuContent = { menuState ->
                                    DropdownMenuItem(
                                        modifier = Modifier.handCursor(),
                                        text = { Text(label) },
                                        onClick = {
                                            onClick()
                                            menuState.dismiss()
                                        },
                                        leadingIcon = { Icon(icon, null, modifier = Modifier.size(18.dp)) }
                                    )
                                }
                            )
                        }
                    }
                }

                Spacer(Modifier.height(24.dp))

                Material3SettingsGroup(
                    items = listOf(
                        Material3SettingsItem(
                            title = { Text("Mark As Read", fontWeight = FontWeight.Bold) },
                            onClick = {
                                guildStore.markGuildAsRead(guild.id)
                                onDismiss()
                            }
                        )
                    )
                )

                Spacer(Modifier.height(24.dp))

                val showChannelsAndRoles = guild.features?.contains("COMMUNITY") == true
                var allowDMs by remember { mutableStateOf(userGuildSettings[guild.id]?.message_notifications != 2) }
                var hideMuted by remember { mutableStateOf(userGuildSettings[guild.id]?.hide_muted_channels == true) }

                val dmItem = switchSettingsItem(
                    title = "Direct Messages",
                    description = "Allow direct messages from server members.",
                    checked = allowDMs,
                    onCheckedChange = {
                        allowDMs = it
                        guildStore.setServerDMsAllowed(guild.id, it)
                    }
                )

                val hideMutedItem = switchSettingsItem(
                    title = "Hide Muted Channels",
                    checked = hideMuted,
                    onCheckedChange = {
                        hideMuted = it
                        guildStore.setHideMutedChannels(guild.id, it)
                    }
                )

                Material3SettingsGroup(
                    items = listOfNotNull(
                        if (showChannelsAndRoles) {
                            Material3SettingsItem(
                                title = { Text("Browse Channels") },
                                onClick = {
                                    navigationStore.isChannelsAndRolesVisible = true
                                    navigationStore.selectedChannel = null
                                    navigationStore.selectedThread = null
                                    onDismiss()
                                }
                            )
                        } else null,

                        Material3SettingsItem(
                            title = { Text("Edit Profile") },
                            description = { Text(currentUser?.global_name ?: currentUser?.username ?: "") },
                            onClick = {
                                navigationStore.navigateToSettings("PROFILES")
                                onDismiss()
                            }
                        ),

                        Material3SettingsItem(
                            title = { Text("Roles") },
                            onClick = { /* TODO: show roles sheet */ }
                        ),

                        dmItem,
                        hideMutedItem,

                        Material3SettingsItem(
                            title = { Text("Leave Server", color = MaterialTheme.colorScheme.error, fontWeight = FontWeight.Bold) },
                            onClick = {
                                showLeaveDialog = true
                            }
                        )
                    )
                )

                if (settingsStore.userSettings?.developer_mode == true) {
                    Spacer(Modifier.height(12.dp))
                    Material3SettingsGroup(
                        items = listOf(
                            Material3SettingsItem(
                                title = { Text("Copy ID") },
                                onClick = {
                                    setClipboardText(guild.id)
                                    onDismiss()
                                }
                            )
                        )
                    )
                }
            }
        }
    }
}
