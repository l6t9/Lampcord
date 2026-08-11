package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.model.OnboardingPrompt
import me.lampu.lampcord.shared.model.OnboardingPromptOption
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.coroutines.launch
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChannelsAndRoles(
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    var selectedTab by remember { mutableStateOf(0) }
    val onboarding = navigationStore.selectedGuildOnboarding
    val guild = navigationStore.selectedGuild
    val currentUser by userStore.currentUser.collectAsState()
    val currentMember = remember(guild?.id, currentUser) {
        if (guild != null && currentUser != null) userStore.getMember(guild.id, currentUser!!.id) else null
    }

    Column(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        PrimaryTabRow(
            selectedTabIndex = selectedTab,
            containerColor = Color.Transparent,
            contentColor = MaterialTheme.colorScheme.primary,
            divider = {}
        ) {
            Tab(
                selected = selectedTab == 0,
                onClick = { selectedTab = 0 },
                text = { Text("Customize") }
            )
            Tab(
                selected = selectedTab == 1,
                onClick = { selectedTab = 1 },
                text = { Text("Browse Channels") }
            )
        }

        Box(modifier = Modifier.weight(1f)) {
            when (selectedTab) {
                0 -> CustomizeTab(navigationStore, onboarding?.prompts ?: emptyList())
                1 -> BrowseChannelsTab(navigationStore, guildStore)
            }
        }
    }
}

@Composable
fun CustomizeTab(navigationStore: NavigationStore, prompts: List<OnboardingPrompt>) {
    if (prompts.isEmpty()) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("No customization options available", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        return
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        items(prompts) { prompt ->
            Column {
                Text(
                    text = prompt.title,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold
                )
                Spacer(Modifier.height(12.dp))
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    prompt.options.forEach { option ->
                        OnboardingOptionItem(option, navigationStore)
                    }
                }
            }
        }
    }
}

@Composable
fun OnboardingOptionItem(
    option: OnboardingPromptOption,
    navigationStore: NavigationStore,
    userStore: UserStore = koinInject()
) {
    // Check if any role in the option is already possessed by the user
    val guild = navigationStore.selectedGuild
    val currentUser by userStore.currentUser.collectAsState()
    val currentMember = remember(guild?.id, currentUser) {
        if (guild != null && currentUser != null) userStore.getMember(guild.id, currentUser!!.id) else null
    }
    var isSelected by remember(option.role_ids, currentMember) {
        mutableStateOf(option.role_ids.any { it in (currentMember?.roles ?: emptyList()) })
    }

    Surface(
        onClick = { 
            isSelected = !isSelected
            // In a real app, this would trigger a role update
        },
        shape = RoundedCornerShape(12.dp),
        color = if (isSelected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceContainerLow,
        border = if (isSelected) androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.primary) else null
    ) {
        Row(
            modifier = Modifier.padding(16.dp).fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (option.emoji?.name != null) {
                Text(option.emoji.name, style = MaterialTheme.typography.headlineSmall)
                Spacer(Modifier.width(16.dp))
            } else if (option.emoji?.id != null) {
                Box(Modifier.size(24.dp).background(Color.Gray, CircleShape))
                Spacer(Modifier.width(16.dp))
            }
            
            Column(modifier = Modifier.weight(1f)) {
                Text(text = option.title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                if (!option.description.isNullOrBlank()) {
                    Text(text = option.description, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            
            Checkbox(checked = isSelected, onCheckedChange = { isSelected = it })
        }
    }
}

@Composable
fun BrowseChannelsTab(navigationStore: NavigationStore, guildStore: GuildStore) {
    val guildId = navigationStore.selectedGuild?.id ?: return
    val allGuildChannels by guildStore.allGuildChannels.collectAsState()
    val allChannels = allGuildChannels.values.filter { it.guild_id == guildId }
    val categories = allChannels.filter { it.type == 4 }.sortedBy { it.position ?: 0 }
    
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        items(categories) { category ->
            val categoryChannels = allChannels.filter { it.parent_id == category.id }.sortedBy { it.position ?: 0 }
            if (categoryChannels.isNotEmpty()) {
                Column {
                    Text(
                        text = category.name?.uppercase() ?: "CHANNELS",
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(Modifier.height(8.dp))
                    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        categoryChannels.forEach { channel ->
                            BrowseChannelItem(channel)
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun BrowseChannelItem(channel: me.lampu.lampcord.shared.model.Channel) {
    val scope = rememberCoroutineScope()
    // In a real implementation, this would check if the channel is currently visible in the sidebar
    var isSelected by remember { mutableStateOf(true) }

    Surface(
        onClick = { 
            isSelected = !isSelected
            scope.launch {
                // TODO: Update opt-in status via API
            }
        },
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Row(
            modifier = Modifier.padding(12.dp).fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = when(channel.type) {
                    2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                    15 -> Icons.Rounded.Forum
                    else -> Icons.Filled.Tag
                },
                contentDescription = null,
                modifier = Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(text = channel.name ?: "unnamed", style = MaterialTheme.typography.bodyLarge)
                if (!channel.topic.isNullOrBlank()) {
                    Text(
                        text = channel.topic,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                    )
                }
            }
            Checkbox(checked = isSelected, onCheckedChange = { isSelected = it })
        }
    }
}
