package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
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
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.utils.setClipboardText

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ServerBottomSheet(
    guild: Guild,
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
        dragHandle = { BottomSheetDefaults.DragHandle() }
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(bottom = 32.dp)
        ) {
            // Header: Banner, Icon, Name
            Box(modifier = Modifier.fillMaxWidth()) {
                val bannerUrl = guild.banner?.let { "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600" }
                if (bannerUrl != null) {
                    AsyncImage(
                        model = bannerUrl,
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(120.dp)
                    )
                } else {
                    Box(modifier = Modifier.fillMaxWidth().height(80.dp).background(MaterialTheme.colorScheme.primaryContainer))
                }

                Surface(
                    modifier = Modifier
                        .padding(start = 16.dp)
                        .offset(y = if (bannerUrl != null) 80.dp else 40.dp)
                        .size(80.dp),
                    shape = RoundedCornerShape(24.dp),
                    color = MaterialTheme.colorScheme.surfaceContainerLow,
                    border = androidx.compose.foundation.BorderStroke(4.dp, MaterialTheme.colorScheme.surfaceContainerLow)
                ) {
                    val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=160" }
                    if (iconUrl != null) {
                        AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                    } else {
                        Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.secondaryContainer), contentAlignment = Alignment.Center) {
                            Text(guild.name?.take(1) ?: "?", style = MaterialTheme.typography.headlineMedium)
                        }
                    }
                }
            }

            Spacer(Modifier.height(if (guild.banner != null) 48.dp else 48.dp))

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
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Box(modifier = Modifier.size(8.dp).background(Color(0xFF23A559), CircleShape))
                            Spacer(Modifier.width(4.dp))
                            Text(
                                "${chatState.onlineCount ?: 0} Online",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Box(modifier = Modifier.size(8.dp).background(Color(0xFFB5BAC1), CircleShape))
                            Spacer(Modifier.width(4.dp))
                            Text(
                                "${chatState.totalMemberCount ?: guild.member_count ?: 0} Members",
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }

                    Spacer(Modifier.height(24.dp))

                    // Top Horizontal Actions (Boost, Notifications, Settings)
                    ButtonGroup(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(64.dp),
                        overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                        horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
                    ) {
                        val actions = listOf(
                            Triple(Icons.Rounded.RocketLaunch, "${guild.premium_subscription_count ?: 0} Boosts", { /* TODO */ }),
                            Triple(Icons.Rounded.Notifications, "Notifications", { /* TODO */ }),
                            Triple(Icons.Rounded.Settings, "Settings", {
                                chatState.isServerSettingsVisible = true
                                onDismiss()
                            })
                        )

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

                // Mark As Read Card
                Material3SettingsGroup(
                    items = listOf(
                        Material3SettingsItem(
                            title = { Text("Mark As Read", fontWeight = FontWeight.Bold) },
                            onClick = {
                                chatState.markGuildAsRead(guild.id)
                                onDismiss()
                            }
                        )
                    )
                )

                Spacer(Modifier.height(24.dp))

                // Options List in a single card
                val showChannelsAndRoles = guild.features?.contains("COMMUNITY") == true
                var allowDMs by remember { mutableStateOf(chatState.userGuildSettingsStore.userGuildSettings[guild.id]?.message_notifications != 2) }
                var hideMuted by remember { mutableStateOf(chatState.userGuildSettingsStore.userGuildSettings[guild.id]?.hide_muted_channels == true) }

                val dmItem = switchSettingsItem(
                    title = "Direct Messages",
                    description = "Allow direct messages from server members.",
                    checked = allowDMs,
                    onCheckedChange = {
                        allowDMs = it
                        chatState.setServerDMsAllowed(guild.id, it)
                    }
                )

                val hideMutedItem = switchSettingsItem(
                    title = "Hide Muted Channels",
                    checked = hideMuted,
                    onCheckedChange = {
                        hideMuted = it
                        chatState.setHideMutedChannels(guild.id, it)
                    }
                )

                Material3SettingsGroup(
                    items = listOfNotNull(
                        if (showChannelsAndRoles) {
                            Material3SettingsItem(
                                title = { Text("Browse Channels") },
                                onClick = {
                                    chatState.isChannelsAndRolesVisible = true
                                    chatState.selectedChannel = null
                                    chatState.selectedThread = null
                                    onDismiss()
                                }
                            )
                        } else null,

                        Material3SettingsItem(
                            title = { Text("Edit Server Profile") },
                            description = { Text(chatState.currentUser?.global_name ?: chatState.currentUser?.username ?: "") },
                            onClick = {
                                chatState.currentUser?.let { chatState.showProfile(it.id) }
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
                                chatState.leaveGuild(guild.id)
                                onDismiss()
                            }
                        )
                    )
                )

                if (chatState.userSettings?.developer_mode == true) {
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
