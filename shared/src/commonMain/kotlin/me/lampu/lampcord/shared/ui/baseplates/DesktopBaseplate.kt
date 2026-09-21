package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandHorizontally
import androidx.compose.animation.shrinkHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.SettingsScreen
import me.lampu.lampcord.shared.ui.components.AttachmentViewer
import me.lampu.lampcord.shared.ui.components.ChannelHeader
import me.lampu.lampcord.shared.ui.components.ChatArea
import me.lampu.lampcord.shared.ui.components.ChatInputBar
import me.lampu.lampcord.shared.ui.components.ChatSkeleton
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.ForumPostList
import me.lampu.lampcord.shared.ui.components.FriendsList
import me.lampu.lampcord.shared.ui.components.MemberList
import me.lampu.lampcord.shared.ui.components.QuickSwitcher
import me.lampu.lampcord.shared.ui.components.Sidebar
import me.lampu.lampcord.shared.ui.components.ThreadPanel
import me.lampu.lampcord.shared.ui.components.VoiceArea
import me.lampu.lampcord.shared.ui.components.chat.SearchScreen
import me.lampu.lampcord.shared.ui.components.guilds.ChannelsAndRoles
import me.lampu.lampcord.shared.ui.components.chat.ChannelSettingsScreen
import me.lampu.lampcord.shared.ui.components.guilds.ServerSettings
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.profiles.UserProfileDialog
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun DesktopBaseplate(
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    voiceStore: VoiceStore = koinInject(),
    widthBreakpoint: WindowWidthBreakpoint = WindowWidthBreakpoint.EXPANDED
) {
    val selectedChannel = navigationStore.selectedChannel
    val selectedThread = navigationStore.selectedThread
    val activeChannel = selectedThread ?: selectedChannel

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surfaceContainerHigh)
            .padding(top = if (getPlatformName() == "windows") 32.dp else 0.dp) // Guild Rail background
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .systemBarsPadding()
                .padding(8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            val sidebarWidth = when (widthBreakpoint) {
                WindowWidthBreakpoint.MEDIUM -> 240.dp
                else -> 312.dp
            }
            Sidebar(modifier = Modifier.width(sidebarWidth))

            // Main Content Area (Chat)
            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                verticalArrangement = Arrangement.Top
            ) {
                Surface(
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.background,
                    tonalElevation = 0.dp
                ) {
                    val quickSpatialSpec = MaterialTheme.motionScheme.defaultSpatialSpec<IntOffset>()
                    val quickEffectsSpec = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()
                    val reduceMotion = Settings.shared.reduceMotion

                    AnimatedContent(
                        targetState = activeChannel?.id
                            ?: if (navigationStore.isChannelsAndRolesVisible) "roles" else if (navigationStore.isFriendsSelected) "friends" else "none",
                        transitionSpec = {
                            if (reduceMotion) {
                                EnterTransition.None togetherWith ExitTransition.None
                            } else {
                                (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                    fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                                )
                            }
                        },
                        modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface),
                        label = "MainContentTransition"
                    ) { target ->
                        if (activeChannel != null && target == activeChannel.id) {
                            val isVoice = activeChannel.type == 2 || activeChannel.type == 13
                            Column(modifier = Modifier.fillMaxSize()) {
                                ChannelHeader(activeChannel, navigationStore, isCompactMemberList = widthBreakpoint == WindowWidthBreakpoint.MEDIUM)
                                
                                Box(modifier = Modifier.weight(1f)) {
                                    if (isVoice) {
                                        if (voiceStore.isVoiceChatTextVisible && widthBreakpoint >= WindowWidthBreakpoint.LARGE) {
                                            Row(Modifier.fillMaxSize()) {
                                                VoiceArea(activeChannel, modifier = Modifier.weight(1f))
                                                Box(Modifier.fillMaxHeight().width(1.dp).background(MaterialTheme.colorScheme.outlineVariant))
                                                ChatArea(modifier = Modifier.width(400.dp))
                                            }
                                        } else if (voiceStore.isVoiceChatTextVisible) {
                                            ChatArea(modifier = Modifier.fillMaxSize())
                                        } else {
                                            VoiceArea(activeChannel)
                                        }
                                    } else if (activeChannel.type == 15 && selectedThread == null) {
                                        ForumPostList()
                                    } else {
                                        ChatArea(
                                            modifier = Modifier.fillMaxSize()
                                        )
                                    }
                                }
                                
                                if (activeChannel.type != 15 && ((!isVoice) || voiceStore.isVoiceChatTextVisible)) {
                                    ChatInputBar(activeChannel)
                                }
                            }
                        } else if (target == "roles") {
                            ChannelsAndRoles()
                        } else if (target == "friends") {
                            FriendsList()
                        } else {
                            ChatUnselectedPlaceholder()
                        }
                    }
                }
            }

            // Member List / Thread Panel (End Panel)
            val showPersistentSidePanel = activeChannel != null && 
                                activeChannel.type != 15 && 
                                (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3) &&
                                widthBreakpoint >= WindowWidthBreakpoint.EXPANDED &&
                                navigationStore.isProfilePanelVisible

            AnimatedVisibility(
                visible = navigationStore.isThreadPanelVisible,
                enter = expandHorizontally(expandFrom = Alignment.End) + fadeIn(),
                exit = shrinkHorizontally(shrinkTowards = Alignment.End) + fadeOut()
            ) {
                Surface(
                    modifier = Modifier.fillMaxHeight().width(340.dp),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.background,
                    tonalElevation = 0.dp
                ) {
                    ThreadPanel()
                }
            }

            AnimatedVisibility(
                visible = showPersistentSidePanel,
                enter = expandHorizontally(expandFrom = Alignment.End) + fadeIn(),
                exit = shrinkHorizontally(shrinkTowards = Alignment.End) + fadeOut()
            ) {
                val endPanelWidth = when (widthBreakpoint) {
                    WindowWidthBreakpoint.EXTRA_LARGE -> 300.dp
                    else -> 240.dp
                }
                Surface(
                    modifier = Modifier.fillMaxHeight().width(endPanelWidth),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.background,
                    tonalElevation = 0.dp
                ) {
                    MemberList()
                }
            }
        }

        // Modal Side Sheet for Medium Breakpoint
        val showModalSidePanel = activeChannel != null && 
                            activeChannel.type != 15 && 
                            (activeChannel.guild_id != null || activeChannel.type == 1 || activeChannel.type == 3) &&
                            widthBreakpoint == WindowWidthBreakpoint.MEDIUM &&
                            navigationStore.isMemberListModalVisible

        AnimatedVisibility(
            visible = showModalSidePanel,
            enter = fadeIn(),
            exit = fadeOut()
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.4f))
                    .clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null
                    ) { navigationStore.isMemberListModalVisible = false }
            ) {
                Surface(
                    modifier = Modifier
                        .fillMaxHeight()
                        .width(320.dp)
                        .align(Alignment.CenterEnd)
                        .padding(8.dp)
                        .clickable(enabled = false) {}
                        .animateEnterExit(
                            enter = slideInHorizontally { it },
                            exit = slideOutHorizontally { it }
                        ),
                    shape = RoundedCornerShape(16.dp),
                    color = MaterialTheme.colorScheme.surface,
                    tonalElevation = 8.dp
                ) {
                    MemberList()
                }
            }
        }

        // Settings / Overlays
        if (navigationStore.isSettingsVisible) {
            SettingsScreen(onDismiss = { navigationStore.isSettingsVisible = false })
        }

        if (navigationStore.isServerSettingsVisible) {
            ServerSettings(onDismiss = { navigationStore.isServerSettingsVisible = false })
        }

        if (navigationStore.channelSettingsChannel != null) {
            ChannelSettingsScreen(onDismiss = { navigationStore.closeChannelSettings() })
        }

        if (navigationStore.isQuickSwitcherVisible) {
            QuickSwitcher(onDismiss = { navigationStore.isQuickSwitcherVisible = false })
        }

        // Attachment Viewer Overlay
        if (navigationStore.isAttachmentViewerVisible) {
            AttachmentViewer(
                items = navigationStore.attachmentViewerItems,
                selectedIndex = navigationStore.attachmentViewerIndex,
                onIndexChange = { navigationStore.attachmentViewerIndex = it },
                onDismiss = { navigationStore.closeAttachmentViewer() }
            )
        }

        // User Profile Dialog
        if (profileStore.isProfileLoading || profileStore.selectedProfile != null) {
            UserProfileDialog(
                profile = profileStore.selectedProfile,
                onDismiss = { 
                    profileStore.selectedProfile = null
                    profileStore.isProfileLoading = false
                }
            )
        }

        // Search Screen
        if (navigationStore.isSearchVisible) {
            SearchScreen(onDismiss = { navigationStore.isSearchVisible = false })
        }
    }
}

@Composable
fun ChatUnselectedPlaceholder(
    navigationStore: NavigationStore = koinInject()
) {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        if (navigationStore.selectedGuild == null) {
            Text("Select a friend to start chatting", color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            ChatSkeleton()
        }
    }
}
