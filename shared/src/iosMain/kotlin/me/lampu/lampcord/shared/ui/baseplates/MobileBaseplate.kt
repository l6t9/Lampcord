package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.ChatArea
import me.lampu.lampcord.shared.ui.components.ChatInputBar
import me.lampu.lampcord.shared.ui.components.GlobalSnackbarHost
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCardSkeleton
import org.koin.compose.koinInject

@Composable
actual fun MobileBaseplate(
    navigationStore: NavigationStore,
    profileStore: ProfileStore,
    userStore: UserStore,
    voiceStore: VoiceStore
) {
    val currentUser by userStore.currentUser.collectAsState()
    val presenceStore: PresenceStore = koinInject()
    val allUsers by userStore.users.collectAsState()

    MaterialTheme {
        Column(modifier = Modifier.fillMaxSize()) {
            val activeChannel = navigationStore.selectedChannel
            val isChat = activeChannel != null && !navigationStore.isChannelsAndRolesVisible

            if (isChat) {
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .statusBarsPadding()
                ) {
                    Box(modifier = Modifier.weight(1f)) {
                        ChatArea(modifier = Modifier.fillMaxSize())
                    }
                    ChatInputBar(activeChannel)
                }
            } else {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .statusBarsPadding(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Select a server to start chatting", style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }

            if (navigationStore.isAttachmentViewerVisible) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text("Attachment Viewer", style = MaterialTheme.typography.bodyLarge)
                }
            }

            if (profileStore.isProfileLoading || profileStore.selectedProfile != null) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.BottomCenter
                ) {
                    if (profileStore.selectedProfile != null) {
                        val profile = profileStore.selectedProfile!!
                        ProfileCard(
                            profile = profile,
                            modifier = Modifier.fillMaxSize(),
                            showBorder = false,
                            isExpanded = true,
                            fillAvailableHeight = true,
                            showBoardTab = true,
                            onDismiss = {
                                profileStore.selectedProfile = null
                                profileStore.isProfileExpanded = false
                                profileStore.isProfileLoading = false
                            }
                        )
                    } else {
                        ProfileCardSkeleton(
                            modifier = Modifier.fillMaxWidth(),
                            isExpanded = true
                        )
                    }
                }
            }

            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.BottomCenter
            ) {
                GlobalSnackbarHost(
                    modifier = Modifier
                        .fillMaxWidth()
                        .navigationBarsPadding()
                        .padding(bottom = 12.dp)
                )
            }
        }
    }
}