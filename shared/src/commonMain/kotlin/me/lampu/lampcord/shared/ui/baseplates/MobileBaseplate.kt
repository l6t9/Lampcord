package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.SettingsScreen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MobileBaseplate(chatState: ChatState) {
    var selectedTab by remember { mutableIntStateOf(0) }
    val selectedChannel = chatState.selectedChannel
    val selectedThread = chatState.selectedThread
    val activeChannel = selectedThread ?: selectedChannel
    
    Scaffold(
        bottomBar = {
            if (activeChannel == null) {
                NavigationBar(
                    containerColor = MaterialTheme.colorScheme.surfaceContainer,
                    tonalElevation = 0.dp
                ) {
                    NavigationBarItem(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        icon = { Icon(Icons.Brand.Discord, "Home") },
                        label = { Text("Home") }
                    )
                    NavigationBarItem(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        icon = { Icon(Icons.Filled.Notifications, "Notifications") },
                        label = { Text("Notifications") }
                    )
                    NavigationBarItem(
                        selected = selectedTab == 2,
                        onClick = { selectedTab = 2 },
                        icon = { Icon(Icons.Filled.AccountCircle, "You") },
                        label = { Text("You") }
                    )
                }
            }
        }
    ) { padding ->
        Box(modifier = Modifier.padding(padding).fillMaxSize()) {
            if (activeChannel != null) {
                Column(modifier = Modifier.fillMaxSize()) {
                    TopAppBar(
                        title = { Text(activeChannel.name ?: "Chat") },
                        navigationIcon = {
                            IconButton(onClick = { chatState.selectedChannel = null; chatState.selectedThread = null }) {
                                Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                            }
                        },
                        colors = TopAppBarDefaults.topAppBarColors(
                            containerColor = MaterialTheme.colorScheme.surfaceContainer
                        )
                    )
                    Box(modifier = Modifier.weight(1f)) {
                         ChatArea(modifier = Modifier.fillMaxSize(), chatState = chatState)
                    }
                    ChatInputBar(activeChannel, chatState)
                }
            } else {
                when (selectedTab) {
                    0 -> {
                        Row(Modifier.fillMaxSize()) {
                            // Mini Guild Rail
                            Column(
                                modifier = Modifier
                                    .width(72.dp)
                                    .fillMaxHeight()
                                    .background(MaterialTheme.colorScheme.surfaceContainerHigh),
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.spacedBy(12.dp),
                                content = {
                                    Spacer(Modifier.height(12.dp))
                                    // Home Icon
                                    val isHomeSelected = chatState.selectedGuild == null
                                    Surface(
                                        modifier = Modifier.size(48.dp),
                                        onClick = { chatState.selectHome() },
                                        shape = androidx.compose.foundation.shape.RoundedCornerShape(if (isHomeSelected) 12.dp else 16.dp),
                                        color = if (isHomeSelected) androidx.compose.ui.graphics.Color(0xFF5865F2) else MaterialTheme.colorScheme.surfaceVariant
                                    ) {
                                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                            Icon(
                                                Icons.Brand.Discord, "Home",
                                                tint = if (isHomeSelected) androidx.compose.ui.graphics.Color.White else MaterialTheme.colorScheme.onSurfaceVariant,
                                                modifier = Modifier.size(28.dp)
                                            )
                                        }
                                    }
                                    HorizontalDivider(modifier = Modifier.width(32.dp))
                                    chatState.guilds.forEach { guild ->
                                        GuildIcon(
                                            guild = guild,
                                            isSelected = chatState.selectedGuild?.id == guild.id,
                                            chatState = chatState,
                                            onClick = { chatState.selectGuild(guild) }
                                        )
                                    }
                                }
                            )
                            // Channel/DM List
                            Box(Modifier.weight(1f).fillMaxHeight().background(MaterialTheme.colorScheme.surfaceContainer)) {
                                if (chatState.selectedGuild != null) {
                                    GuildChannelList(chatState)
                                } else {
                                    DMList(chatState)
                                }
                            }
                        }
                    }
                    1 -> {
                        NotificationList(chatState = chatState)
                    }
                    2 -> {
                        SettingsScreen(chatState = chatState, onDismiss = { selectedTab = 0 })
                    }
                }
            }
        }
    }
}
