package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.*
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.components.guilds.*
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.profiles.UserProfileDialog
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.SettingsScreen

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun DesktopBaseplate(chatState: ChatState) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surface)
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown) {
                    if (event.isCtrlPressed && event.key == Key.K) {
                        chatState.isQuickSwitcherVisible = true
                        return@onPreviewKeyEvent true
                    }
                }
                false
            }
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(6.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            Sidebar(chatState, Modifier.width(318.dp))

            // Main Content Area (Chat)
            val selectedChannel = chatState.selectedChannel
            val selectedThread = chatState.selectedThread
            val activeChannel = selectedThread ?: selectedChannel

            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Surface(
                    modifier = Modifier.weight(1f),
                    shape = MaterialTheme.shapes.large,
                    tonalElevation = 2.dp
                ) {
                    val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
                    val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

                    AnimatedContent(
                        targetState = if (activeChannel != null) activeChannel.id else if (chatState.isChannelsAndRolesVisible) "roles" else if (chatState.isFriendsSelected) "friends" else "none",
                        transitionSpec = {
                            (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                            )
                        },
                        label = "MainContentTransition"
                    ) { target ->
                        if (activeChannel != null && target == activeChannel.id) {
                            Column(modifier = Modifier.fillMaxSize()) {
                                ChannelHeader(activeChannel, chatState)
                                
                                Box(modifier = Modifier.weight(1f)) {
                                    if ((activeChannel.type == 2 || activeChannel.type == 13) && !chatState.isVoiceChatTextVisible) {
                                        VoiceArea(activeChannel, chatState)
                                    } else if (activeChannel.type == 15 && selectedThread == null) {
                                        ForumPostList(chatState)
                                    } else {
                                        ChatArea(
                                            modifier = Modifier.fillMaxSize(),
                                            chatState = chatState
                                        )
                                    }
                                }
                                
                                if (activeChannel.type != 15 && ((activeChannel.type != 2 && activeChannel.type != 13) || chatState.isVoiceChatTextVisible)) {
                                    ChatInputBar(activeChannel, chatState)
                                }
                            }
                        } else if (target == "roles") {
                            ChannelsAndRoles(chatState)
                        } else if (target == "friends") {
                            FriendsList(chatState)
                        } else {
                            ChatUnselectedPlaceholder(chatState)
                        }
                    }
                }
            }

            // Member List / Profile (End Panel)
            val showMemberList = activeChannel?.guild_id != null && activeChannel.type != 15
            val showDMProfile = activeChannel?.type == 1

            if (showMemberList || showDMProfile) {
                Column(
                    modifier = Modifier
                        .width(if (showDMProfile) 340.dp else 240.dp)
                        .fillMaxHeight(),
                    verticalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Surface(
                        modifier = Modifier.weight(1f),
                        shape = MaterialTheme.shapes.medium,
                        tonalElevation = 1.dp
                    ) {
                        if (showDMProfile) {
                            val profile = chatState.sidebarProfile
                            if (profile != null) {
                                ProfileCard(
                                    profile = profile,
                                    chatState = chatState,
                                    showBorder = true,
                                    isSidebar = true,
                                    showMemberSince = true,
                                    modifier = Modifier.fillMaxSize(),
                                    onExpand = { chatState.showProfile(profile.user.id) }
                                )
                            } else if (chatState.isSidebarProfileLoading) {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    ContainedLoadingIndicator()
                                }
                            }
                        } else {
                            MemberList(chatState)
                        }
                    }
                    
                    // HomeNavButtons equivalent
                    if (showMemberList) {
                        NavButtonRow(
                            listOf(
                                NavButtonData(
                                    icon = Icons.Filled.Group,
                                    title = "Members",
                                    selected = chatState.isFriendsSelected,
                                    onClick = {
                                        chatState.selectedGuild = null
                                        chatState.selectedChannel = null
                                        chatState.isFriendsSelected = true
                                    }
                                ),
                                NavButtonData(
                                    icon = Icons.Filled.Search,
                                    title = "Search",
                                    onClick = { chatState.isQuickSwitcherVisible = true }
                                ),
                                NavButtonData(
                                    icon = Icons.Outlined.AlternateEmail,
                                    title = "Mentions",
                                    onClick = { /* Mentions */ }
                                )
                            )
                        )
                    }
                }
            }
        }

        // Settings / Overlays
        if (chatState.isSettingsVisible) {
            SettingsScreen(chatState, onDismiss = { chatState.isSettingsVisible = false })
        }

        if (chatState.isServerSettingsVisible) {
            ServerSettings(chatState, onDismiss = { chatState.isServerSettingsVisible = false })
        }
        
        if (chatState.isQuickSwitcherVisible) {
            QuickSwitcher(chatState, onDismiss = { chatState.isQuickSwitcherVisible = false })
        }

        // Attachment Viewer Overlay
        if (chatState.isAttachmentViewerVisible) {
            AttachmentViewer(
                items = chatState.attachmentViewerItems,
                selectedIndex = chatState.attachmentViewerIndex,
                onIndexChange = { chatState.attachmentViewerIndex = it },
                onDismiss = { chatState.closeAttachmentViewer() }
            )
        }

        // User Profile Dialog
        if (chatState.isProfileLoading || chatState.selectedProfile != null) {
            UserProfileDialog(
                profile = chatState.selectedProfile,
                chatState = chatState,
                onDismiss = { 
                    chatState.selectedProfile = null
                    chatState.isProfileLoading = false
                }
            )
        }
    }
}

@Composable
fun ChatUnselectedPlaceholder(chatState: ChatState) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        if (chatState.selectedGuild == null) {
            Text("Select a friend to start chatting", color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            ChatSkeleton()
        }
    }
}
