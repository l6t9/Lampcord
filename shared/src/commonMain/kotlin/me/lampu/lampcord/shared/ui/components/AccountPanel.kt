@file:OptIn(ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.theme.*
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun AccountPanel(chatState: ChatState) {
    val user = chatState.currentUser ?: return
    
    var panelPosition by remember { mutableStateOf(androidx.compose.ui.geometry.Offset.Zero) }
    var showStatusMenu by remember { mutableStateOf(false) }
    var showAccountPicker by remember { mutableStateOf(false) }
    var showAddAccountDialog by remember { mutableStateOf(false) }
    var showCustomStatusDialog by remember { mutableStateOf(false) }
    
    val isMuted = chatState.currentVoiceState?.self_mute ?: false
    val isDeafened = chatState.currentVoiceState?.self_deaf ?: false

    if (showCustomStatusDialog) {
        CustomStatusDialog(
            initialText = chatState.userSettings?.custom_status?.text ?: "",
            onDismiss = { showCustomStatusDialog = false },
            onSave = { text ->
                chatState.updateCustomStatus(text.ifBlank { null })
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
        val nameplate = chatState.currentMember?.collectibles?.nameplate ?: user.collectibles?.nameplate
        
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
                val member = chatState.currentMember
                val avatarUrl = member?.avatar?.let {
                    "https://cdn.discordapp.com/guilds/${chatState.selectedGuild?.id}/users/${user.id}/avatars/$it.png?size=160"
                } ?: user.avatar?.let {
                    "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160"
                }

                Box(modifier = Modifier.size(32.dp)) {
                    val status = chatState.userSettings?.status ?: "online"
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
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = user.username ?: "",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
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
                                onCheckedChange = { chatState.toggleVoiceMute() },
                                shapes = ButtonGroupDefaults.connectedLeadingButtonShapes(),
                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                contentPadding = PaddingValues(0.dp),
                                modifier = Modifier.fillMaxSize()
                            ) {
                                Icon(
                                    imageVector = if (isMuted) Icons.Filled.MicOff else Icons.Filled.Mic,
                                    contentDescription = "Mute",
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                        },
                        menuContent = {
                            DropdownMenuItem(
                                text = { Text(if (isMuted) "Unmute" else "Mute") },
                                onClick = { chatState.toggleVoiceMute() },
                                leadingIcon = { Icon(if (isMuted) Icons.Filled.MicOff else Icons.Filled.Mic, null, modifier = Modifier.size(18.dp)) }
                            )
                        }
                    )
                    customItem(
                        buttonGroupContent = {
                            ToggleButton(
                                checked = isDeafened, 
                                onCheckedChange = { chatState.toggleVoiceDeaf() },
                                shapes = ButtonGroupDefaults.connectedMiddleButtonShapes(),
                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                contentPadding = PaddingValues(0.dp),
                                modifier = Modifier.fillMaxSize()
                            ) {
                                Icon(
                                    imageVector = if (isDeafened) Icons.Filled.HeadsetOff else Icons.Filled.Headphones,
                                    contentDescription = "Deafen",
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                        },
                        menuContent = {
                            DropdownMenuItem(
                                text = { Text(if (isDeafened) "Undeafen" else "Deafen") },
                                onClick = { chatState.toggleVoiceDeaf() },
                                leadingIcon = { Icon(if (isDeafened) Icons.Filled.HeadsetOff else Icons.Filled.Headphones, null, modifier = Modifier.size(18.dp)) }
                            )
                        }
                    )
                    customItem(
                        buttonGroupContent = {
                            ToggleButton(
                                checked = false,
                                onCheckedChange = { chatState.isSettingsVisible = true },
                                shapes = ButtonGroupDefaults.connectedTrailingButtonShapes(),
                                colors = ToggleButtonDefaults.tonalToggleButtonColors(),
                                contentPadding = PaddingValues(0.dp),
                                modifier = Modifier.fillMaxSize()
                            ) {
                                Icon(Icons.Filled.Settings, "Settings", modifier = Modifier.size(18.dp))
                            }
                        },
                        menuContent = {
                            DropdownMenuItem(
                                text = { Text("Settings") },
                                onClick = { chatState.isSettingsVisible = true },
                                leadingIcon = { Icon(Icons.Filled.Settings, null) }
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
                        chatState.showProfile(user.id, panelPosition)
                    },
                    leadingIcon = { Icon(Icons.Filled.Person, null) }
                )
                
                DropdownMenuItem(
                    text = { Text("Edit Custom Status") },
                    onClick = {
                        showStatusMenu = false
                        showCustomStatusDialog = true
                    },
                    leadingIcon = { Icon(Icons.Filled.Edit, null) }
                )

                DropdownMenuItem(
                    text = { Text("Switch Account") },
                    onClick = {
                        showStatusMenu = false
                        showAccountPicker = true
                    },
                    leadingIcon = { Icon(Icons.Filled.Groups, null) }
                )

                DropdownMenuItem(
                    text = { Text("Settings") },
                    onClick = {
                        showStatusMenu = false
                        chatState.isSettingsVisible = true
                    },
                    leadingIcon = { Icon(Icons.Filled.Settings, null) }
                )
                
                HorizontalDivider()
                
                DropdownMenuItem(
                    text = { Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(Modifier.size(10.dp).background(DiscordGreen, CircleShape))
                        Spacer(Modifier.width(8.dp))
                        Text("Online")
                    }},
                    onClick = {
                        chatState.updateStatus("online")
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
                        chatState.updateStatus("idle")
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
                        chatState.updateStatus("dnd")
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
                        chatState.updateStatus("invisible")
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
                        chatState = chatState,
                        onAccountSelected = { account ->
                            if (account.user.id != user.id) {
                                chatState.disconnect()
                                chatState.connect(account.token)
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
        title = { Text("Set a custom status") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("What's bubbling?", style = MaterialTheme.typography.labelMedium)
                OutlinedTextField(
                    value = text,
                    onValueChange = { text = it },
                    modifier = Modifier.fillMaxWidth(),
                    placeholder = { Text("Support for statuses is here!") },
                    singleLine = true
                )
            }
        },
        confirmButton = {
            TextButton(onClick = { onSave(text) }) {
                Text("Save")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        }
    )
}
