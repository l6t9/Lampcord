@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.snap
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AlertDialogDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.api.CdnUrls
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.ToggleButton
import androidx.compose.material3.ToggleButtonDefaults
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.handCursor
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@Composable
fun AccountPanel(
    userStore: UserStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    sessionManager: SessionManager = koinInject()
) {
    val currentUser by userStore.currentUser.collectAsState()
    val user = currentUser ?: return
    
    val userSettings = settingsStore.userSettings
    val scope = rememberCoroutineScope()
    
    var panelPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
    var showStatusMenu by remember { mutableStateOf(false) }
    var showAccountPicker by remember { mutableStateOf(false) }
    var showAddAccountDialog by remember { mutableStateOf(false) }
    var showCustomStatusDialog by remember { mutableStateOf(false) }
    
    val isMuted = voiceStore.selfMuted
    val isDeafened = voiceStore.selfDeafened

    if (showCustomStatusDialog) {
        CustomStatusDialog(
            initialText = userSettings?.custom_status?.text ?: "",
            onDismiss = { showCustomStatusDialog = false },
            onSave = { text ->
                scope.launch {
                    presenceStore.updateCustomStatus(text.ifBlank { null })
                }
                showCustomStatusDialog = false
            }
        )
    }

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .height(68.dp)
            .handCursor()
            .onGloballyPositioned { panelPosition = it.positionInRoot() },
        color = MaterialTheme.colorScheme.surfaceContainer,
        tonalElevation = 4.dp
    ) {
        val member = navigationStore.selectedGuild?.let { userStore.getMember(it.id, user.id) }
        val nameplate = member?.collectibles?.nameplate ?: user.collectibles?.nameplate
        var isHovered by remember { mutableStateOf(false) }
        
        Box(
            modifier = Modifier
                .fillMaxSize()
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
        ) {
            if (nameplate != null) {
                val decoUrl = "https://cdn.discordapp.com/assets/collectibles/${nameplate.asset}img.png?passthrough=true"
                AsyncImage(
                    model = decoUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                    alpha = 0.4f,
                    allowAnimation = isHovered
                )
            }

            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .clickableCursor { showStatusMenu = true }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val avatarUrl = member?.avatar?.let {
                    "https://cdn.discordapp.com/guilds/${navigationStore.selectedGuild?.id}/users/${user.id}/avatars/$it.png?size=160"
                } ?: CdnUrls.getUserAvatarUrl(user.id, user.avatar, 160)

                Box(modifier = Modifier.size(32.dp)) {
                    val status = userSettings?.status ?: "online"
                    AvatarWithDecoration(
                        avatarUrl = avatarUrl,
                        decorationData = member?.avatar_decoration_data ?: user.avatar_decoration_data ?: member?.collectibles?.avatar_decoration ?: user.collectibles?.avatar_decoration,
                        size = 32.dp,
                        status = status,
                        forceAnimate = true
                    )
                }
                
                Spacer(modifier = Modifier.width(10.dp))
                
                val roleData = remember(member, navigationStore.selectedGuild) {
                    val m = member ?: return@remember null
                    val g = navigationStore.selectedGuild ?: return@remember null
                    val colorRole = m.getRoleColorRole(g)
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

                Column(modifier = Modifier.weight(1f)) {
                    UsernameView(
                        name = member?.nick ?: user.global_name ?: user.username ?: "Unknown",
                        style = member?.display_name_styles ?: user.display_name_styles,
                        baseStyle = MaterialTheme.typography.labelLarge,
                        fontWeight = androidx.compose.ui.text.font.FontWeight.Bold,
                        color = if (roleColor != Color.Unspecified) roleColor else MaterialTheme.colorScheme.onSurface,
                        roleGradient = roleGradient,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        marquee = true
                    )
                    
                    val presences by presenceStore.presences.collectAsState()
                    val presence = presences[user.id]
                    val activity = presence?.activities?.firstOrNull()

                    if (activity != null) {
                        UserActivity(
                            activity = activity,
                            compact = true
                        )
                    } else {
                        Text(
                            text = user.username ?: "",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = if (!Settings.shared.marqueeEnabled) Modifier else Modifier.basicMarquee(
                                iterations = if (getPlatformName() == "windows") 1 else Int.MAX_VALUE,
                                initialDelayMillis = 3000,
                                velocity = 30.dp
                            )
                        )
                    }
                }
                
                ButtonGroup(
                    modifier = Modifier
                        .height(32.dp)
                        .animateContentSize(
                            animationSpec = if (Settings.shared.reduceMotion) {
                                snap()
                            } else {
                                spring(dampingRatio = 0.6f, stiffness = 400f)
                            }
                        ),
                    overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                    horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
                ) {
                    customItem(
                        buttonGroupContent = {
                            ToggleButton(
                                checked = isMuted, 
                                onCheckedChange = { voiceStore.toggleVoiceMute() },
                                shapes = ButtonGroupDefaults.connectedLeadingButtonShapes(),
                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                contentPadding = PaddingValues(0.dp),
                                modifier = Modifier.fillMaxSize()
                            ) {
                                Icon(
                                    imageVector = if (isMuted) Icons.Filled.MicOff else Icons.Rounded.Mic,
                                    contentDescription = "Mute",
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                        },
                        menuContent = {
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text(if (isMuted) "Unmute" else "Mute") },
                                onClick = { voiceStore.toggleVoiceMute() },
                                leadingIcon = { Icon(if (isMuted) Icons.Filled.MicOff else Icons.Rounded.Mic, null, modifier = Modifier.size(18.dp)) }
                            )
                        }
                    )
                    customItem(
                        buttonGroupContent = {
                            ToggleButton(
                                checked = isDeafened, 
                                onCheckedChange = { voiceStore.toggleVoiceDeaf() },
                                shapes = ButtonGroupDefaults.connectedMiddleButtonShapes(),
                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                contentPadding = PaddingValues(0.dp),
                                modifier = Modifier.fillMaxSize()
                            ) {
                                Icon(
                                    imageVector = if (isDeafened) Icons.Filled.HeadsetOff else Icons.Rounded.Headphones,
                                    contentDescription = "Deafen",
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                        },
                        menuContent = {
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text(if (isDeafened) "Undeafen" else "Deafen") },
                                onClick = { voiceStore.toggleVoiceDeaf() },
                                leadingIcon = { Icon(if (isDeafened) Icons.Filled.HeadsetOff else Icons.Rounded.Headphones, null, modifier = Modifier.size(18.dp)) }
                            )
                        }
                    )
                    customItem(
                        buttonGroupContent = {
                            ToggleButton(
                                checked = false,
                                onCheckedChange = { navigationStore.isSettingsVisible = true },
                                shapes = ButtonGroupDefaults.connectedTrailingButtonShapes(),
                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                contentPadding = PaddingValues(0.dp),
                                modifier = Modifier.fillMaxSize()
                            ) {
                                Icon(Icons.Rounded.Settings, "Settings", modifier = Modifier.size(18.dp))
                            }
                        },
                        menuContent = {
                            DropdownMenuItem(
                                modifier = Modifier.handCursor(),
                                text = { Text("Settings") },
                                onClick = { navigationStore.isSettingsVisible = true },
                                leadingIcon = { Icon(Icons.Rounded.Settings, null) }
                            )
                        }
                    )
                }
            }

            DropdownMenu(
                expanded = showStatusMenu,
                onDismissRequest = { showStatusMenu = false },
                modifier = Modifier.width(220.dp)
            ) {
                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Text("View Profile") },
                    onClick = {
                        showStatusMenu = false
                        profileStore.showProfile(user.id, navigationStore.selectedGuild?.id, panelPosition)
                    },
                    leadingIcon = { Icon(Icons.Rounded.Person, null) }
                )
                
                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Text("Edit Custom Status") },
                    onClick = {
                        showStatusMenu = false
                        showCustomStatusDialog = true
                    },
                    leadingIcon = { Icon(Icons.Rounded.Edit, null) }
                )

                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Text("Switch Account") },
                    onClick = {
                        showStatusMenu = false
                        showAccountPicker = true
                    },
                    leadingIcon = { Icon(Icons.Rounded.Groups, null) }
                )

                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Text("Settings") },
                    onClick = {
                        showStatusMenu = false
                        navigationStore.isSettingsVisible = true
                    },
                    leadingIcon = { Icon(Icons.Rounded.Settings, null) }
                )
                
                HorizontalDivider()
                
                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        StatusIndicator(status = "online", size = 12.dp, borderWidth = 0.dp)
                        Spacer(Modifier.width(8.dp))
                        Text("Online")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("online") }
                        showStatusMenu = false
                    }
                )
                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        StatusIndicator(status = "idle", size = 12.dp, borderWidth = 0.dp)
                        Spacer(Modifier.width(8.dp))
                        Text("Idle")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("idle") }
                        showStatusMenu = false
                    }
                )
                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        StatusIndicator(status = "dnd", size = 12.dp, borderWidth = 0.dp)
                        Spacer(Modifier.width(8.dp))
                        Text("Do Not Disturb")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("dnd") }
                        showStatusMenu = false
                    }
                )
                DropdownMenuItem(
                    modifier = Modifier.handCursor(),
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        StatusIndicator(status = "invisible", size = 12.dp, borderWidth = 0.dp)
                        Spacer(Modifier.width(8.dp))
                        Text("Invisible")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("invisible") }
                        showStatusMenu = false
                    }
                )
            }

            if (showAccountPicker) {
                androidx.compose.ui.window.Popup(
                    onDismissRequest = { showAccountPicker = false },
                    alignment = Alignment.BottomStart,
                    offset = IntOffset(16, (-70).dp.value.toInt()),
                    properties = androidx.compose.ui.window.PopupProperties(focusable = true)
                ) {
                    AccountPicker(
                        tokenStore = sessionManager.tokenStore,
                        userStore = userStore,
                        onAccountSelected = { account ->
                            if (account.user.id != user.id) {
                                sessionManager.switchAccount(account.token)
                            }
                            showAccountPicker = false
                        },
                        onAddAccount = {
                            showAccountPicker = false
                            showAddAccountDialog = true
                        }
                    )
                }
            }

            if (showAddAccountDialog) {
                androidx.compose.ui.window.Dialog(
                    onDismissRequest = { showAddAccountDialog = false },
                    properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
                ) {
                    Surface(
                        modifier = Modifier.fillMaxSize(),
                        color = MaterialTheme.colorScheme.background
                    ) {
                        me.lampu.lampcord.shared.ui.LoginScreen(onLoginSuccess = { showAddAccountDialog = false })
                    }
                }
            }
        }
    }
}

@Composable
fun CustomStatusDialog(
    initialText: String,
    onDismiss: () -> Unit,
    onSave: (String) -> Unit
) {
    var text by remember { mutableStateOf(initialText) }

    AlertDialog(
        onDismissRequest = onDismiss,
        icon = {
            Icon(
                Icons.Filled.Edit,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = MaterialTheme.colorScheme.primary
            )
        },
        title = {
            Text(
                text = "Set a custom status",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = androidx.compose.ui.text.font.FontWeight.Bold,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        },
        text = {
            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Text(
                    text = "What's bubbling?",
                    style = MaterialTheme.typography.bodyMedium,
                    textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                
                OutlinedTextField(
                    value = text,
                    onValueChange = { text = it },
                    modifier = Modifier.fillMaxWidth(),
                    placeholder = { Text("Support for statuses is here!") },
                    singleLine = true,
                    shape = MaterialTheme.shapes.medium
                )
            }
        },
        confirmButton = {
            ExpressiveTooltip(
                tooltipString = "Save changes",
                anchorPosition = TooltipAnchorPosition.End,
                content = {
                    tooltipText("Save changes")
                }
            )
            Button(
                onClick = { onSave(text) },
                modifier = Modifier.fillMaxWidth(),
                shape = CircleShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.primary,
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                )
            ) {
                Text("Save")
            }
        },
        dismissButton = {
            ExpressiveTooltip(
                tooltipString = "Close without saving",
                anchorPosition = TooltipAnchorPosition.End,
                content = {
                    tooltipText("Close without saving")
                }
            )
            TextButton(
                onClick = onDismiss,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Cancel")
            }
        },
        shape = MaterialTheme.shapes.extraLarge,
        tonalElevation = AlertDialogDefaults.TonalElevation
    )
}
