package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.input.nestedscroll.nestedScroll
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MentionsStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MentionsScreen(
    mentionsStore: MentionsStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    messageStore: me.lampu.lampcord.shared.state.MessageStore = koinInject(),
    channelNavigator: me.lampu.lampcord.shared.state.ChannelNavigator = koinInject()
) {
    LaunchedEffect(Unit) {
        mentionsStore.loadMentions(refresh = true)
    }

    val allChannels by guildStore.allGuildChannels.collectAsState()
    val privateChannels by guildStore.privateChannels.collectAsState()
    val guilds by guildStore.guilds.collectAsState()

    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()

    Scaffold(
        modifier = Modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        topBar = {
            LargeTopAppBar(
                title = { Text("Mentions") },
                navigationIcon = {
                    IconButton(onClick = { navigationStore.isMentionsSelected = false }) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                scrollBehavior = scrollBehavior
            )
        }
    ) { padding ->
        if (mentionsStore.isLoading && mentionsStore.mentions.isEmpty()) {
            Box(Modifier.fillMaxSize().padding(padding), contentAlignment = Alignment.Center) {
                ContainedLoadingIndicator()
            }
        } else if (mentionsStore.mentions.isEmpty()) {
            Box(Modifier.fillMaxSize().padding(padding), contentAlignment = Alignment.Center) {
                Text("No mentions yet", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(padding),
                contentPadding = PaddingValues(bottom = 80.dp)
            ) {
                items(mentionsStore.mentions) { message ->
                    val chan = allChannels[message.channel_id] ?: privateChannels.find { it.id == message.channel_id }
                    val guild = guilds.firstOrNull { it.id == message.guild_id }

                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickableCursor {
                                navigationStore.isMentionsSelected = false
                                navigationStore.isFriendsSelected = false
                                val targetChan = chan ?: privateChannels.find { it.id == message.channel_id }
                                if (targetChan != null) {
                                    if (guild != null) {
                                        navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) }
                                    }
                                    navigationStore.selectChannel(targetChan, explicitlySelected = true)
                                    messageStore.scrollToMessageId = message.id
                                } else {
                                    channelNavigator.navigateToChannel(message.channel_id, message.guild_id)
                                    messageStore.scrollToMessageId = message.id
                                }
                            }
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = chan?.name ?: "unknown-channel",
                                style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                                color = MaterialTheme.colorScheme.onSurface
                            )
                            if (guild != null) {
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    text = guild.name ?: "unknown-server",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                )
                            }
                        }

                        Surface(
                            modifier = Modifier.padding(horizontal = 8.dp),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f)
                        ) {
                            MessageItem(message = message)
                        }
                        
                        Spacer(Modifier.height(8.dp))
                    }
                }
            }
        }
    }
}
