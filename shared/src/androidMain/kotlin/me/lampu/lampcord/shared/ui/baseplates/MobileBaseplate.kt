package me.lampu.lampcord.shared.ui.baseplates

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.SettingsScreen
import io.github.materiiapps.panels.SwipePanels
import io.github.materiiapps.panels.SwipePanelsValue
import io.github.materiiapps.panels.rememberSwipePanelsState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
actual fun MobileBaseplate(chatState: ChatState) {
    val panelState = rememberSwipePanelsState()
    val selectedChannel = chatState.selectedChannel
    val selectedThread = chatState.selectedThread
    val activeChannel = selectedThread ?: selectedChannel

    BackHandler(enabled = panelState.currentValue != SwipePanelsValue.Center) {
        panelState.close()
    }

    Box(Modifier.fillMaxSize()) {
        SwipePanels(
            state = panelState,
            inBetweenPadding = 6.dp,
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.surface),
            start = {
                Sidebar(
                    chatState = chatState,
                    modifier = Modifier
                        .fillMaxSize()
                        .systemBarsPadding()
                        .padding(start = 6.dp)
                )
            },
            center = {
                val centerPanelShape = if (panelState.currentValue != SwipePanelsValue.Center) {
                    MaterialTheme.shapes.large
                } else {
                    RoundedCornerShape(0.dp)
                }

                Surface(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(centerPanelShape),
                    color = MaterialTheme.colorScheme.surface,
                    tonalElevation = 2.dp
                ) {
                    Scaffold(
                        topBar = {
                            if (activeChannel != null) {
                                TopAppBar(
                                    title = {
                                        Text(activeChannel.name ?: "Chat", style = MaterialTheme.typography.titleMedium)
                                    },
                                    navigationIcon = {
                                        IconButton(onClick = { panelState.openStart() }) {
                                            Icon(Icons.Filled.Menu, "Channels")
                                        }
                                    },
                                    actions = {
                                        if (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3) {
                                            IconButton(onClick = { panelState.openEnd() }) {
                                                Icon(Icons.Filled.Group, "Members")
                                            }
                                        }
                                    },
                                    colors = TopAppBarDefaults.topAppBarColors(
                                        containerColor = MaterialTheme.colorScheme.surface
                                    )
                                )
                            }
                        }
                    ) { padding ->
                        Box(Modifier.padding(padding).fillMaxSize()) {
                            if (activeChannel != null) {
                                Column(modifier = Modifier.fillMaxSize()) {
                                    Box(modifier = Modifier.weight(1f)) {
                                        ChatArea(modifier = Modifier.fillMaxSize(), chatState = chatState)
                                    }
                                    ChatInputBar(activeChannel, chatState)
                                }
                            } else if (chatState.isFriendsSelected) {
                                FriendsList(chatState)
                            } else {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                        Icon(Icons.Brand.Discord, null, modifier = Modifier.size(80.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.2f))
                                        Spacer(Modifier.height(24.dp))
                                        Text("Select a channel to start chatting", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        Spacer(Modifier.height(24.dp))
                                        Button(onClick = { panelState.openStart() }) {
                                            Text("Open Drawer")
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if (panelState.currentValue != SwipePanelsValue.Center) {
                        Box(
                            modifier = Modifier
                                .zIndex(1f)
                                .fillMaxSize()
                                .clickable(
                                    interactionSource = remember { MutableInteractionSource() },
                                    indication = null,
                                    onClick = { panelState.close() },
                                )
                        )
                    }
                }
            },
            end = {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .systemBarsPadding()
                        .padding(end = 6.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Surface(
                        modifier = Modifier.weight(1f),
                        shape = MaterialTheme.shapes.medium,
                        tonalElevation = 1.dp
                    ) {
                        MemberList(chatState)
                    }
                    
                    Row(
                        modifier = Modifier.height(60.dp).fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        NavButton(Icons.Filled.Group) { /* Friends */ }
                        NavButton(Icons.Filled.Search) { /* Search */ }
                        NavButton(Icons.Outlined.AlternateEmail) { /* Mentions */ }
                    }
                }
            }
        )

        if (chatState.isAttachmentViewerVisible) {
            AttachmentViewer(
                items = chatState.attachmentViewerItems,
                selectedIndex = chatState.attachmentViewerIndex,
                onIndexChange = { chatState.attachmentViewerIndex = it },
                onDismiss = { chatState.closeAttachmentViewer() }
            )
        }

        // User Profile Sheet
        if (chatState.isProfileLoading || chatState.selectedProfile != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    chatState.selectedProfile = null
                    chatState.isProfileLoading = false
                },
                shape = RoundedCornerShape(topStart = 32.dp, topEnd = 32.dp),
                containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
                dragHandle = {
                    Box(
                        modifier = Modifier
                            .padding(vertical = 12.dp)
                            .size(width = 40.dp, height = 4.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
                    )
                }
            ) {
                if (chatState.selectedProfile != null) {
                    ProfileCard(
                        profile = chatState.selectedProfile!!,
                        chatState = chatState,
                        modifier = Modifier.fillMaxWidth().wrapContentHeight(),
                        showBorder = false,
                        isExpanded = true
                    )
                } else {
                    Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator()
                    }
                }
                Spacer(Modifier.navigationBarsPadding().height(16.dp))
            }
        }
    }
}
