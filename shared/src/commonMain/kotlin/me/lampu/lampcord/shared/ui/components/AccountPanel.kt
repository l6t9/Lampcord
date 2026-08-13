@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
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
import me.lampu.lampcord.shared.ui.theme.DiscordGray
import me.lampu.lampcord.shared.ui.theme.DiscordGreen
import me.lampu.lampcord.shared.ui.theme.DiscordRed
import me.lampu.lampcord.shared.ui.theme.DiscordYellow
import org.koin.compose.koinInject

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
    
    val currentVoiceState = voiceStore.currentVoiceState
    val isMuted = currentVoiceState?.self_mute ?: false
    val isDeafened = currentVoiceState?.self_deaf ?: false

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
            .onGloballyPositioned { panelPosition = it.positionInRoot() },
        color = MaterialTheme.colorScheme.surfaceContainer,
        tonalElevation = 4.dp
    ) {
        val member = navigationStore.selectedGuild?.let { userStore.getMember(it.id, user.id) }
        val nameplate = member?.collectibles?.nameplate ?: user.collectibles?.nameplate
        
        Box(modifier = Modifier.fillMaxSize()) {
            if (nameplate != null) {
                val decoUrl = "https://cdn.discordapp.com/assets/collectibles/${nameplate.asset}img.png?passthrough=true"
                AsyncImage(
                    model = decoUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                    alpha = 0.4f
                )
            }

            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .clickable { showStatusMenu = true }
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val avatarUrl = member?.avatar?.let {
                    "https://cdn.discordapp.com/guilds/${navigationStore.selectedGuild?.id}/users/${user.id}/avatars/$it.png?size=160"
                } ?: user.avatar?.let {
                    "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160"
                }

                Box(modifier = Modifier.size(32.dp)) {
                    val status = userSettings?.status ?: "online"
                    AvatarWithDecoration(
                        avatarUrl = avatarUrl,
                        decorationData = member?.avatar_decoration_data ?: user.avatar_decoration_data ?: member?.collectibles?.avatar_decoration ?: user.collectibles?.avatar_decoration,
                        size = 32.dp,
                        status = status
                    )
                }
                
                Spacer(modifier = Modifier.width(10.dp))
                
                Column(modifier = Modifier.weight(1f)) {
                    UsernameView(
                        name = member?.nick ?: user.global_name ?: user.username ?: "Unknown",
                        style = member?.display_name_styles ?: user.display_name_styles,
                        baseStyle = MaterialTheme.typography.labelLarge,
                        fontWeight = androidx.compose.ui.text.font.FontWeight.Bold,
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
                            modifier = Modifier.basicMarquee(iterations = Int.MAX_VALUE, initialDelayMillis = 3000, velocity = 30.dp)
                        )
                    }
                }
                
                ButtonGroup(
                    modifier = Modifier
                        .height(32.dp)
                        .animateContentSize(animationSpec = spring(dampingRatio = 0.6f, stiffness = 400f)),
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
                    text = { Text("View Profile") },
                    onClick = {
                        showStatusMenu = false
                        profileStore.showProfile(user.id, navigationStore.selectedGuild?.id, panelPosition)
                    },
                    leadingIcon = { Icon(Icons.Rounded.Person, null) }
                )
                
                DropdownMenuItem(
                    text = { Text("Edit Custom Status") },
                    onClick = {
                        showStatusMenu = false
                        showCustomStatusDialog = true
                    },
                    leadingIcon = { Icon(Icons.Rounded.Edit, null) }
                )

                DropdownMenuItem(
                    text = { Text("Switch Account") },
                    onClick = {
                        showStatusMenu = false
                        showAccountPicker = true
                    },
                    leadingIcon = { Icon(Icons.Rounded.Groups, null) }
                )

                DropdownMenuItem(
                    text = { Text("Settings") },
                    onClick = {
                        showStatusMenu = false
                        navigationStore.isSettingsVisible = true
                    },
                    leadingIcon = { Icon(Icons.Rounded.Settings, null) }
                )
                
                HorizontalDivider()
                
                DropdownMenuItem(
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(Modifier.size(10.dp).background(DiscordGreen, CircleShape))
                        Spacer(Modifier.width(8.dp))
                        Text("Online")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("online") }
                        showStatusMenu = false
                    }
                )
                DropdownMenuItem(
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(Modifier.size(10.dp).background(DiscordYellow, CircleShape))
                        Spacer(Modifier.width(8.dp))
                        Text("Idle")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("idle") }
                        showStatusMenu = false
                    }
                )
                DropdownMenuItem(
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(Modifier.size(10.dp).background(DiscordRed, CircleShape))
                        Spacer(Modifier.width(8.dp))
                        Text("Do Not Disturb")
                    }},
                    onClick = {
                        scope.launch { presenceStore.updateStatus("dnd") }
                        showStatusMenu = false
                    }
                )
                DropdownMenuItem(
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(Modifier.size(10.dp).background(DiscordGray, CircleShape))
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
                shape = MaterialTheme.shapes.medium
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
