package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.VoiceStore
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
import me.lampu.lampcord.shared.ui.components.VoiceArea
import me.lampu.lampcord.shared.ui.components.chat.SearchScreen
import me.lampu.lampcord.shared.ui.components.guilds.ChannelsAndRoles
import me.lampu.lampcord.shared.ui.components.guilds.ServerSettings
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.profiles.UserProfileDialog
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun DesktopBaseplate(
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    voiceStore: VoiceStore = koinInject()
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surface)
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .systemBarsPadding()
                .padding(6.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            Sidebar(modifier = Modifier.width(312.dp))

            // Main Content Area (Chat)
            val selectedChannel = navigationStore.selectedChannel
            val selectedThread = navigationStore.selectedThread
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
                        targetState = activeChannel?.id
                            ?: if (navigationStore.isChannelsAndRolesVisible) "roles" else if (navigationStore.isFriendsSelected) "friends" else "none",
                        transitionSpec = {
                            (fadeIn(quickEffectsSpec) + slideInHorizontally(quickSpatialSpec) { it / 8 }).togetherWith(
                                fadeOut(quickEffectsSpec) + slideOutHorizontally(quickSpatialSpec) { -it / 8 }
                            )
                        },
                        modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface),
                        label = "MainContentTransition"
                    ) { target ->
                        if (activeChannel != null && target == activeChannel.id) {
                            Column(modifier = Modifier.fillMaxSize()) {
                                ChannelHeader(activeChannel, navigationStore)
                                
                                Box(modifier = Modifier.weight(1f)) {
                                    if ((activeChannel.type == 2 || activeChannel.type == 13) && !voiceStore.isVoiceChatTextVisible) {
                                        VoiceArea(activeChannel)
                                    } else if (activeChannel.type == 15 && selectedThread == null) {
                                        ForumPostList()
                                    } else {
                                        ChatArea(
                                            modifier = Modifier.fillMaxSize()
                                        )
                                    }
                                }
                                
                                if (activeChannel.type != 15 && ((activeChannel.type != 2 && activeChannel.type != 13) || voiceStore.isVoiceChatTextVisible)) {
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
                    // HomeNavButtons equivalent
                    if (showMemberList) {
                        Surface(
                            onClick = { navigationStore.isSearchVisible = true },
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(40.dp),
                            shape = RoundedCornerShape(20.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f),
                            tonalElevation = 1.dp
                        ) {
                            Row(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .padding(horizontal = 12.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Icon(
                                    imageVector = Icons.Filled.Search,
                                    contentDescription = null,
                                    modifier = Modifier.size(18.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = "Search",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                )
                            }
                        }
                    }

                    Surface(
                        modifier = Modifier.weight(1f),
                        shape = MaterialTheme.shapes.medium,
                        tonalElevation = 1.dp
                    ) {
                        if (showDMProfile) {
                            val profile = profileStore.sidebarProfile
                            if (profile != null) {
                                ProfileCard(
                                    profile = profile,
                                    showBorder = true,
                                    isSidebar = true,
                                    showMemberSince = true,
                                    modifier = Modifier.fillMaxSize(),
                                    onExpand = { profileStore.showProfile(profile.user.id, navigationStore.selectedGuild?.id) }
                                )
                            } else if (profileStore.isSidebarProfileLoading) {
                                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                    ContainedLoadingIndicator()
                                }
                            }
                        } else {
                            MemberList()
                        }
                    }
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
