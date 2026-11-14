package me.lampu.lampcord.shared.ui.components.members

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.search
import me.lampu.lampcord.shared.generated.resources.close_dm
import me.lampu.lampcord.shared.generated.resources.copy_id
import me.lampu.lampcord.shared.generated.resources.pinned_messages
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.AutocompleteStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SearchStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.CleanUtils
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.ui.kit.handCursor

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MemberHeader(
    channel: Channel,
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject(),
    searchStore: SearchStore = koinInject(),
    autocompleteStore: AutocompleteStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    onBack: (() -> Unit)? = null,
    overlapping: Boolean = false,
) {
    val currentUser by userStore.currentUser.collectAsState()
    val allUsers by userStore.users.collectAsState()
    val guild = navigationStore.selectedGuild
    val member = remember(guild?.id, currentUser) {
        if (guild != null && currentUser != null) userStore.getMember(guild.id, currentUser!!.id) else null
    }

    val canManageChannel = remember(channel, guild, member) {
        if (guild == null || member == null) false
        else PermissionHelper.hasPermission(member, guild, channel, Permission.MANAGE_CHANNELS, currentUser?.id)
    }

    val platform = getPlatformName()
    val isDesktop = platform == "desktop" || platform == "macos" || platform == "windows" || platform == "linux"

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.surface)
            .padding(
                top = if (overlapping) 4.dp else 16.dp,
                bottom = if (overlapping) 4.dp else 8.dp
            )
    ) {
        if (!isDesktop && overlapping) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(44.dp)
                    .padding(horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                if (onBack != null) {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
                Spacer(Modifier.weight(1f))
                IconButton(onClick = { navigationStore.isSearchVisible = true }) {
                    Icon(
                        imageVector = Icons.Filled.Search,
                        contentDescription = "Search",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                IconButton(onClick = { navigationStore.isPinsVisible = true }) {
                    Icon(
                        imageVector = Icons.Filled.PushPin,
                        contentDescription = "Pins",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                if (canManageChannel) {
                    IconButton(onClick = { navigationStore.openChannelSettings(channel) }) {
                        Icon(
                            imageVector = Icons.Filled.Settings,
                            contentDescription = "Settings",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
        }

        if (isDesktop) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp)
                    .height(40.dp)
                    .clip(RoundedCornerShape(20.dp))
                    .background(MaterialTheme.colorScheme.surfaceContainerHigh)
                    .clickable { navigationStore.isSearchVisible = true }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Filled.Search,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(Modifier.width(10.dp))
                Text(
                    text = stringResource(Res.string.search),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        } else {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                val icon = if (channel.type == 3) {
                    Icons.Rounded.Groups
                } else if (isDm) {
                    Icons.Rounded.AlternateEmail
                } else {
                    when (channel.type) {
                        15 -> Icons.Rounded.Forum
                        2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                        5 -> Icons.Filled.Campaign
                        else -> Icons.Filled.Tag
                    }
                }

                val name = remember(channel, allUsers, isDm, currentUser) {
                    if (isDm) {
                        if (channel.type == 3) {
                            if (!channel.name.isNullOrBlank()) {
                                channel.name
                            } else {
                                val recipients = channel.recipients?.mapNotNull { allUsers[it.id] ?: it }
                                    ?: channel.recipient_ids?.mapNotNull { allUsers[it] }
                                    ?: emptyList()
                                val otherRecipients = if (currentUser != null) recipients.filter { it.id != currentUser?.id } else recipients
                                val displayList = if (otherRecipients.isNotEmpty()) otherRecipients else recipients
                                displayList.mapNotNull { it.global_name ?: it.username }
                                    .joinToString(", ")
                                    .ifEmpty { "Unnamed Group DM" }
                            }
                        } else {
                            val recipientId = channel.recipients?.firstOrNull()?.id
                                ?: channel.recipient_ids?.firstOrNull()
                            val recipient = recipientId?.let { allUsers[it] }
                                ?: channel.recipients?.firstOrNull()
                            recipient?.let { it.global_name ?: it.username } ?: "Unknown"
                        }
                    } else {
                        CleanUtils.cleanChannelName(channel.name ?: "unnamed")
                    }
                }

                val isPrivate = remember(channel, guild) {
                    if (guild == null) false
                    else PermissionHelper.isChannelPrivate(guild, channel)
                }

                if (!isDm || channel.type == 3) {
                    Box(modifier = Modifier.size(24.dp)) {
                        Icon(
                            imageVector = icon,
                            contentDescription = null,
                            modifier = Modifier.fillMaxSize(),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )

                        if (isPrivate && channel.type != 4 && !isDm) {
                            Surface(
                                modifier = Modifier
                                    .align(Alignment.TopEnd)
                                    .offset(x = 2.dp, y = (-2).dp)
                                    .size(12.dp),
                                shape = CircleShape,
                                color = MaterialTheme.colorScheme.surface,
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Lock,
                                    contentDescription = null,
                                    modifier = Modifier.padding(1.dp).fillMaxSize(),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                    }
                    Spacer(Modifier.width(12.dp))
                }
                Text(
                    text = name,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f)
                )

                if (isDm) {
                    var showMenu by remember { mutableStateOf(false) }
                    Box {
                        IconButton(onClick = { showMenu = true }) {
                            Icon(
                                imageVector = Icons.Filled.MoreVert,
                                contentDescription = "More options",
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }

                        DropdownMenu(
                            expanded = showMenu,
                            onDismissRequest = { showMenu = false }
                        ) {
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text(stringResource(Res.string.close_dm)) },
                                onClick = {
                                    showMenu = false
                                    navigationStore.closeDm(channel.id)
                                },
                                leadingIcon = { Icon(Icons.Filled.Close, null, tint = Color.Red) }
                            )
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text(stringResource(Res.string.pinned_messages)) },
                                onClick = {
                                    showMenu = false
                                    navigationStore.isPinsVisible = true
                                },
                                leadingIcon = { Icon(Icons.Filled.PushPin, null) }
                            )
                            if (settingsStore.userSettings?.developer_mode == true) {
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
                                    text = { Text(stringResource(Res.string.copy_id)) },
                                    onClick = {
                                        showMenu = false
                                        me.lampu.lampcord.shared.utils.setClipboardText(channel.id)
                                    },
                                    leadingIcon = { Icon(Icons.Filled.Dns, null) }
                                )
                            }
                        }
                    }
                }
            }

            if (channel.topic?.isNotBlank() == true) {
                Spacer(Modifier.height(4.dp))
                var expanded by remember { mutableStateOf(false) }
                Text(
                    text = channel.topic,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = if (expanded) Int.MAX_VALUE else 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .padding(horizontal = 16.dp)
                        .clickableCursor { expanded = !expanded }
                )
            }
        }

        Spacer(Modifier.height(if (overlapping) 4.dp else 16.dp))

        if (!isDesktop && !overlapping) {
            Spacer(Modifier.height(12.dp))

            ButtonGroup(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp),
                overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
            ) {
                val allActions = listOf(
                    HeaderButtonData(
                        icon = Icons.Rounded.Search,
                        label = "Search",
                        onClick = { navigationStore.isSearchVisible = true }
                    ),
                    HeaderButtonData(
                        icon = Icons.Rounded.PushPin,
                        label = "Pins",
                        onClick = { navigationStore.isPinsVisible = true }
                    ),
                    HeaderButtonData(
                        icon = Icons.Rounded.Settings,
                        label = "Settings",
                        onClick = { navigationStore.openChannelSettings(channel) },
                        enabled = channel.type != 1 && channel.type != 3 && channel.guild_id != null && canManageChannel
                    )
                )

                val actions = allActions.filter { it.enabled }

                actions.forEachIndexed { index, action ->
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
                                onClick = action.onClick,
                                enabled = action.enabled,
                                shapes = shapes,
                                modifier = Modifier
                                    .weight(1f)
                                    .heightIn(min = 56.dp),
                                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 10.dp)
                            ) {
                                val contentColor = if (action.enabled) {
                                    MaterialTheme.colorScheme.onSecondaryContainer
                                } else {
                                    MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f)
                                }
                                Column(
                                    horizontalAlignment = Alignment.CenterHorizontally,
                                    verticalArrangement = Arrangement.spacedBy(4.dp, Alignment.CenterVertically)
                                ) {
                                    Icon(
                                        imageVector = action.icon,
                                        contentDescription = action.label,
                                        modifier = Modifier.size(24.dp),
                                        tint = contentColor
                                    )
                                    Text(
                                        text = action.label,
                                        style = MaterialTheme.typography.labelMedium,
                                        color = contentColor,
                                        textAlign = TextAlign.Center,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                        },
                        menuContent = { menuState ->
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text(action.label) },
                                onClick = {
                                    action.onClick()
                                    menuState.dismiss()
                                },
                                leadingIcon = { Icon(action.icon, null, modifier = Modifier.size(18.dp)) },
                                enabled = action.enabled
                            )
                        }
                    )
                }
            }
        }
    }
}

private data class HeaderButtonData(
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val label: String,
    val onClick: () -> Unit,
    val enabled: Boolean = true
)
