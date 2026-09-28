package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.state.AutocompleteStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SearchStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
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
            val focusManager = LocalFocusManager.current
            var isFocused by remember { mutableStateOf(false) }
            var textFieldValue by remember {
                mutableStateOf<TextFieldValue>(TextFieldValue(searchStore.searchQuery, TextRange(searchStore.searchQuery.length)))
            }

            LaunchedEffect(searchStore.searchQuery) {
                if (searchStore.searchQuery != textFieldValue.text) {
                    textFieldValue = TextFieldValue(searchStore.searchQuery, TextRange(searchStore.searchQuery.length))
                }
            }

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp)
                    .height(40.dp)
                    .clip(RoundedCornerShape(20.dp))
                    .background(
                        if (isFocused) MaterialTheme.colorScheme.surfaceContainerHigh 
                        else MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.65f)
                    )
                    .border(
                        1.dp, 
                        if (isFocused) MaterialTheme.colorScheme.primary.copy(alpha = 0.5f) 
                        else Color.Transparent, 
                        RoundedCornerShape(20.dp)
                    )
            ) {
                BasicTextField(
                    value = textFieldValue,
                    onValueChange = { value ->
                        textFieldValue = value
                        searchStore.searchQuery = value.text
                        if (value.text.isNotBlank()) {
                            val lastPart = value.text.split(" ").last()
                            val (type, query) = when {
                                lastPart.startsWith("from:", ignoreCase = true) -> AutocompleteType.USER to lastPart.substring(5)
                                lastPart.startsWith("mentions:", ignoreCase = true) -> AutocompleteType.USER to lastPart.substring(9)
                                lastPart.startsWith("in:", ignoreCase = true) -> AutocompleteType.CHANNEL to lastPart.substring(3)
                                lastPart.startsWith("@") -> AutocompleteType.MENTION to lastPart.substring(1)
                                lastPart.startsWith("#") -> AutocompleteType.CHANNEL to lastPart.substring(1)
                                else -> null to ""
                            }
                            autocompleteStore.updateAutocomplete(type, query, navigationStore.selectedGuild, channel, isSearch = true)
                        } else {
                            autocompleteStore.clear(isSearch = true)
                        }
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .align(Alignment.CenterStart)
                        .onFocusChanged { isFocused = it.isFocused },
                    textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                    keyboardActions = KeyboardActions(onSearch = {
                        if (searchStore.searchQuery.isNotBlank()) {
                            searchStore.performSearch(navigationStore.selectedGuild, navigationStore.selectedChannel)
                            navigationStore.isSearchVisible = true
                            focusManager.clearFocus()
                        }
                    }),
                    decorationBox = { innerTextField: @Composable () -> Unit ->
                        Row(
                            modifier = Modifier.padding(horizontal = 12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(
                                imageVector = Icons.Filled.Search,
                                contentDescription = null,
                                modifier = Modifier.size(18.dp),
                                tint = if (isFocused) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                            )
                            Spacer(Modifier.width(10.dp))
                            Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                                if (searchStore.searchQuery.isEmpty()) {
                                    Text(
                                        "Search in ${channel.name ?: "channel"}",
                                        style = MaterialTheme.typography.bodyMedium,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                    )
                                }
                                innerTextField()
                            }
                            if (searchStore.searchQuery.isNotEmpty()) {
                                IconButton(
                                    onClick = { 
                                        searchStore.searchQuery = ""
                                        autocompleteStore.clear(isSearch = true)
                                    },
                                    modifier = Modifier.size(24.dp)
                                ) {
                                    Icon(
                                        Icons.Filled.Close,
                                        contentDescription = "Clear",
                                        modifier = Modifier.size(16.dp),
                                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                            }
                        }
                    }
                )

                DropdownMenu(
                    expanded = isFocused && autocompleteStore.searchAutocompleteType != null && autocompleteStore.searchAutocompleteItems.isNotEmpty(),
                    onDismissRequest = { /* Don't hide here to keep focus */ },
                    properties = androidx.compose.ui.window.PopupProperties(focusable = false),
                    modifier = Modifier.width(300.dp).background(MaterialTheme.colorScheme.surfaceContainerHigh),
                    offset = DpOffset(16.dp, 4.dp)
                ) {
                    autocompleteStore.searchAutocompleteItems.forEach { item ->
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = {
                                Text(
                                    item.title,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            },
                            leadingIcon = item.icon?.let { icon ->
                                {
                                    AsyncImage(
                                        model = icon,
                                        contentDescription = null,
                                        modifier = Modifier.size(24.dp).clip(CircleShape)
                                    )
                                }
                            },
                            onClick = {
                                val currentQuery = textFieldValue.text
                                val parts = currentQuery.split(" ").toMutableList()
                                if (parts.isNotEmpty()) {
                                    val lastPart = parts.last()
                                    val prefix = if (lastPart.contains(":")) lastPart.substringBefore(":") + ":" else ""
                                    val replacement = item.searchReplacement ?: item.replacement
                                    parts[parts.lastIndex] = prefix + replacement
                                    val newText = parts.joinToString(" ") + " "
                                    textFieldValue = TextFieldValue(newText, TextRange(newText.length))
                                    searchStore.searchQuery = newText
                                }
                                autocompleteStore.clear(isSearch = true)
                            }
                        )
                    }
                }
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

                val isPrivate = remember(channel, guild) {
                    if (guild == null) false
                    else PermissionHelper.isChannelPrivate(guild, channel)
                }

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
                                text = { Text("Close DM") },
                                onClick = {
                                    showMenu = false
                                    navigationStore.closeDm(channel.id)
                                },
                                leadingIcon = { Icon(Icons.Filled.Close, null, tint = Color.Red) }
                            )
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text("Pinned Messages") },
                                onClick = {
                                    showMenu = false
                                    navigationStore.isPinsVisible = true
                                },
                                leadingIcon = { Icon(Icons.Filled.PushPin, null) }
                            )
                            if (settingsStore.userSettings?.developer_mode == true) {
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
                                    text = { Text("Copy ID") },
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

        Spacer(Modifier.height(16.dp))
        
        if (!isDesktop) {
            Spacer(Modifier.height(16.dp))

            ButtonGroup(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp)
                    .height(64.dp),
                overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
            ) {
                val allActions = listOf(
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
                        icon = Icons.Filled.Settings,
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
