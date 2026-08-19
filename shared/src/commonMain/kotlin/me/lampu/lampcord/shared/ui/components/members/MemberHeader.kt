package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.state.AutocompleteStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SearchStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.CleanUtils
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MemberHeader(
    channel: Channel,
    navigationStore: NavigationStore = koinInject(),
    userStore: UserStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    searchStore: SearchStore = koinInject(),
    autocompleteStore: AutocompleteStore = koinInject()
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
            .padding(top = 16.dp, bottom = 8.dp)
    ) {
        if (isDesktop) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp)
                    .height(40.dp)
                    .background(
                        MaterialTheme.colorScheme.surfaceContainerHigh,
                        RoundedCornerShape(20.dp)
                    )
                    .padding(horizontal = 12.dp),
                contentAlignment = Alignment.CenterStart
            ) {
                if (searchStore.searchQuery.isEmpty()) {
                    Text(
                        "Search in ${channel.name ?: "channel"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                    )
                }
                BasicTextField(
                    value = searchStore.searchQuery,
                    onValueChange = {
                        searchStore.searchQuery = it
                        if (it.isNotBlank()) {
                            val lastPart = it.split(" ").last()
                            val (type, query) = when {
                                lastPart.startsWith("from:", ignoreCase = true) -> AutocompleteType.USER to lastPart.substring(5)
                                lastPart.startsWith("mentions:", ignoreCase = true) -> AutocompleteType.USER to lastPart.substring(9)
                                lastPart.startsWith("in:", ignoreCase = true) -> AutocompleteType.CHANNEL to lastPart.substring(3)
                                lastPart.startsWith("@") -> AutocompleteType.MENTION to lastPart.substring(1)
                                lastPart.startsWith("#") -> AutocompleteType.CHANNEL to lastPart.substring(1)
                                else -> null to ""
                            }
                            autocompleteStore.updateAutocomplete(type, query, navigationStore.selectedGuild, isSearch = true)
                        } else {
                            autocompleteStore.clear(isSearch = true)
                        }
                    },
                    modifier = Modifier.fillMaxWidth(),
                    textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                    keyboardActions = KeyboardActions(onSearch = {
                        if (searchStore.searchQuery.isNotBlank()) {
                            searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                            navigationStore.isSearchVisible = true
                        }
                    })
                )
            }
        } else {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val isDm = channel.type == 1 || channel.type == 3 || channel.guild_id == null
                val icon = if (isDm) {
                    Icons.Rounded.AlternateEmail
                } else {
                    when (channel.type) {
                        15 -> Icons.Rounded.Forum
                        2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                        5 -> Icons.Filled.Campaign
                        else -> Icons.Filled.Tag
                    }
                }
                
                val name = remember(channel, allUsers, isDm) {
                    if (isDm) {
                        val recipientId = channel.recipients?.firstOrNull()?.id
                            ?: channel.recipient_ids?.firstOrNull()
                        val recipient = recipientId?.let { allUsers[it] }
                            ?: channel.recipients?.firstOrNull()
                        if (channel.name?.isNotBlank() == true) {
                            channel.name
                        } else {
                            recipient?.let { it.global_name ?: it.username } ?: "Unknown"
                        }
                    } else {
                        CleanUtils.cleanChannelName(channel.name ?: "unnamed")
                    }
                }

                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(24.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(Modifier.width(12.dp))
                Text(
                    text = name,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }

            if (channel.topic?.isNotBlank() == true) {
                Spacer(Modifier.height(4.dp))
                Text(
                    text = channel.topic,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(horizontal = 16.dp)
                )
            }
        }

        Spacer(Modifier.height(16.dp))
        
        HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
        
        Spacer(Modifier.height(16.dp))

        ButtonGroup(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp)
                .height(64.dp),
            overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
            horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
        ) {
            val actions = listOf(
                HeaderButtonData(
                    icon = Icons.Filled.Search,
                    label = "Search",
                    onClick = { navigationStore.isSearchVisible = true }
                ),
                HeaderButtonData(
                    icon = Icons.Filled.PushPin,
                    label = "Pins",
                    onClick = { navigationStore.isPinsVisible = true }
                ),
                HeaderButtonData(
                    icon = Icons.Filled.Notifications,
                    label = "Notifications",
                    onClick = { navigationStore.isNotificationsSettingsVisible = true }
                ),
                HeaderButtonData(
                    icon = Icons.Filled.Settings,
                    label = "Settings",
                    onClick = { navigationStore.openChannelSettings(channel) },
                    enabled = channel.type != 1 && channel.type != 3 && channel.guild_id != null && canManageChannel
                )
            )

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
                            modifier = Modifier.weight(1f).fillMaxHeight(),
                            contentPadding = PaddingValues(0.dp)
                        ) {
                            val contentAlpha = if (action.enabled) 1f else 0.4f
                            Column(
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.Center
                            ) {
                                Icon(
                                    imageVector = action.icon,
                                    contentDescription = null,
                                    modifier = Modifier.size(20.dp).alpha(contentAlpha)
                                )
                                Spacer(Modifier.height(4.dp))
                                Text(
                                    text = action.label,
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSecondaryContainer.copy(alpha = contentAlpha),
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

private data class HeaderButtonData(
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val label: String,
    val onClick: () -> Unit,
    val enabled: Boolean = true
)
