package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
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
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Onboarding
import me.lampu.lampcord.shared.model.OnboardingPrompt
import me.lampu.lampcord.shared.model.OnboardingPromptOption
import me.lampu.lampcord.shared.model.customEmojiCdnUrl
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private const val CATEGORY = 4
private val VOICE_TYPES = setOf(2, 13)
private val FORUM_TYPES = setOf(15, 16)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChannelsAndRoles(
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    userStore: UserStore = koinInject(),
    channelApi: ChannelApi = koinInject(),
) {
    var selectedTab by remember { mutableStateOf(0) }
    val guild = navigationStore.selectedGuild
    val onboarding by remember(guild?.id) {
        derivedStateOf { guild?.let { navigationStore.selectedGuildOnboarding } }
    }

    Column(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        PrimaryTabRow(
            selectedTabIndex = selectedTab,
            containerColor = Color.Transparent,
            contentColor = MaterialTheme.colorScheme.primary,
            divider = {}
        ) {
            Tab(selected = selectedTab == 0, onClick = { selectedTab = 0 }, text = { Text("Customize") })
            Tab(selected = selectedTab == 1, onClick = { selectedTab = 1 }, text = { Text("Browse Channels") })
            Tab(selected = selectedTab == 2, onClick = { selectedTab = 2 }, text = { Text("Roles") })
        }

        Box(modifier = Modifier.weight(1f)) {
            val onboardingPrompts = onboarding?.prompts ?: emptyList()
            when (selectedTab) {
                0 -> CustomizeTab(
                    guild = guild,
                    onboarding = onboarding,
                    onSave = { optionIds ->
                        val id = guild?.id ?: return@CustomizeTab false
                        channelApi.saveOnboardingResponses(id, optionIds, onboardingPrompts, initial = true)
                    }
                )

                1 -> BrowseChannelsTab(guildId = guild?.id, channelApi = channelApi)

                2 -> RolesTab(guild = guild)
            }
        }
    }
}

@Composable
private fun CustomizeTab(
    guild: me.lampu.lampcord.shared.model.Guild?,
    onboarding: Onboarding?,
    onSave: suspend (Set<String>) -> Boolean
) {
    val prompts = onboarding?.prompts.orEmpty()
    var selected by remember(onboarding) {
        mutableStateOf(prompts.flatMap { it.options.map { o -> o.id } }.toMutableSet())
    }
    val scope = rememberCoroutineScope()
    var saving by remember { mutableStateOf(false) }
    var status by remember { mutableStateOf<String?>(null) }

    if (prompts.isEmpty()) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                text = if (onboarding?.enabled == true) {
                    "This server has no customization prompts."
                } else {
                    "Onboarding is not set up for this server."
                },
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        return
    }

    Column(modifier = Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            items(prompts, key = { it.id }) { prompt ->
                Column {
                    Text(
                        text = prompt.title,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold
                    )
                    if (prompt.required) {
                        Text(
                            text = "Required",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    Spacer(Modifier.height(12.dp))
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        prompt.options.forEach { option ->
                            val isOn = option.id in selected
                            OnboardingOptionItem(
                                option = option,
                                checked = isOn,
                                onCheckedChange = { wanted ->
                                    selected = selected.toggle(option, prompt, wanted)
                                }
                            )
                        }
                    }
                }
            }
        }

        Surface(tonalElevation = 3.dp) {
            Column(Modifier.padding(16.dp)) {
                status?.let {
                    Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.height(8.dp))
                }
                Button(
                    onClick = {
                        saving = true
                        status = null
                        scope.launch {
                            val guildName = guild?.name
                            val ok = onSave(selected)
                            saving = false
                            status = if (ok) {
                                "Saved. You may need to restart to see the changes in $guildName."
                            } else {
                                "Could not save your answers."
                            }
                        }
                    },
                    enabled = !saving,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    if (saving) {
                        CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 2.dp)
                    } else {
                        Text("Save")
                    }
                }
            }
        }
    }
}

private fun MutableSet<String>.toggle(
    option: OnboardingPromptOption,
    prompt: OnboardingPrompt,
    wanted: Boolean
): MutableSet<String> {
    val next = toMutableSet()
    if (wanted) {
        if (prompt.single_select) {
            val siblings = prompt.options.map { it.id }
            next.removeAll(siblings.toSet())
        }
        next.add(option.id)
    } else {
        if (prompt.required) {
            val remaining = prompt.options.count { it.id in next }
            if (remaining <= 1) return this
        }
        next.remove(option.id)
    }
    return next
}

@Composable
private fun OnboardingOptionItem(
    option: OnboardingPromptOption,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Surface(
        onClick = { onCheckedChange(!checked) },
        shape = RoundedCornerShape(12.dp),
        color = if (checked) {
            MaterialTheme.colorScheme.primaryContainer
        } else {
            MaterialTheme.colorScheme.surfaceContainerLow
        },
        border = if (checked) {
            androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.primary)
        } else {
            null
        }
    ) {
        Row(
            modifier = Modifier.padding(16.dp).fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            OptionEmoji(option.emoji)
            Spacer(Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(option.title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold)
                if (!option.description.isNullOrBlank()) {
                    Text(
                        option.description,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            Checkbox(checked = checked, onCheckedChange = onCheckedChange)
        }
    }
}

@Composable
private fun OptionEmoji(emoji: Emoji?) {
    val name = emoji?.name
    val id = emoji?.id
    when {
        !name.isNullOrBlank() -> Text(name, style = MaterialTheme.typography.headlineSmall)
        !id.isNullOrBlank() -> AsyncEmojiImage(id, size = 24)
        else -> Box(Modifier.size(24.dp).background(Color.Gray, CircleShape))
    }
}

@Composable
private fun AsyncEmojiImage(id: String, size: Int) {
    val animated = id.startsWith("a_")
    val numeric = id.removePrefix("a_")
    AsyncImage(
        model = customEmojiCdnUrl(numeric, animated, size = size * 2),
        contentDescription = null,
        modifier = Modifier.size(size.dp),
        filterQuality = androidx.compose.ui.graphics.FilterQuality.Medium,
        allowAnimation = animated,
        showPlaceholder = false
    )
}@Composable
private fun BrowseChannelsTab(
    guildId: String?,
    channelApi: ChannelApi = koinInject(),
    guildStore: GuildStore = koinInject()
) {
    val scope = rememberCoroutineScope()
    var channels by remember(guildId) { mutableStateOf<List<Channel>>(emptyList()) }
    var loaded by remember(guildId) { mutableStateOf(false) }

    LaunchedEffect(guildId) {
        val id = guildId ?: return@LaunchedEffect
        val fetched = withContext(Dispatchers.IO) {
            runCatching { channelApi.getGuildChannels(id) }.getOrDefault(emptyList())
        }
        channels = fetched
        loaded = true
    }

    if (!loaded) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        return
    }
    if (channels.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                "No channels to show.",
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        return
    }

    val browsable = channels.filter { it.type != CATEGORY && it.type != 1 && it.type != 3 }
    val order = compareBy<Channel>(
        { if (it.type in VOICE_TYPES || it.type in FORUM_TYPES) 1 else 0 },
        { it.position ?: 0 }
    )
    val rows = buildList {
        val roots = browsable.filter { it.parent_id == null }.sortedWith(order)
        if (roots.isNotEmpty()) {
            add(Row.Header("Channels"))
            roots.forEach { add(Row.ChannelRow(it)) }
        }
        channels.filter { it.type == CATEGORY }
            .sortedBy { it.position ?: 0 }
            .forEach { category ->
                val children = browsable.filter { it.parent_id == category.id }.sortedWith(order)
                if (children.isNotEmpty()) {
                    add(Row.Header(category.name ?: "Channels"))
                    children.forEach { add(Row.ChannelRow(it)) }
                }
            }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(rows, key = { it.id }) { row ->
            when (row) {
                is Row.Header -> ChannelGroupHeader(row.title)
                is Row.ChannelRow -> BrowseChannelItem(
                    channel = row.channel,
                    initiallyOptedIn = guildId?.let { guildStore.isChannelOptedIn(it, row.channel.id) } ?: true,
                    onOptInChange = { wanted ->
                        val id = guildId
                        if (id != null) guildStore.setChannelOptIn(id, row.channel.id, wanted)
                    }
                )
            }
        }
    }
}

private sealed interface Row {
    val id: String

    data class Header(val title: String) : Row {
        override val id: String get() = "header:$title"
    }

    data class ChannelRow(val channel: Channel) : Row {
        override val id: String get() = "channel:${channel.id}"
    }
}

@Composable
private fun ChannelGroupHeader(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.onSurfaceVariant
    )
}

@Composable
private fun BrowseChannelItem(
    channel: Channel,
    initiallyOptedIn: Boolean,
    onOptInChange: (Boolean) -> Unit
) {
    var optedIn by remember(channel.id, initiallyOptedIn) { mutableStateOf(initiallyOptedIn) }
    fun toggle(to: Boolean) {
        optedIn = to
        onOptInChange(to)
    }
    Surface(
        onClick = { toggle(!optedIn) },
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Row(
            modifier = Modifier.padding(12.dp).fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = when (channel.type) {
                    in VOICE_TYPES -> Icons.AutoMirrored.Filled.VolumeUp
                    in FORUM_TYPES -> Icons.Rounded.Forum
                    else -> Icons.Filled.Tag
                },
                contentDescription = null,
                modifier = Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(channel.name ?: "unnamed", style = MaterialTheme.typography.bodyLarge)
                if (!channel.topic.isNullOrBlank()) {
                    Text(
                        channel.topic,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                    )
                }
            }
            Checkbox(checked = optedIn, onCheckedChange = ::toggle)
        }
    }
}

@Composable
private fun RolesTab(
    guild: me.lampu.lampcord.shared.model.Guild?,
    navigationStore: NavigationStore = koinInject()
) {
    val guildId = guild?.id
    val roles = remember(guildId) { guild?.roles.orEmpty() }
    val assignable = roles
        .filter { role -> role.permissions.isBlank() || !role.permissions.managesRoles() }
        .sortedByDescending { it.position }

    if (assignable.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                "This server has no roles you can assign to yourself.",
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
        return
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        items(assignable, key = { it.id }) { role ->
            RoleRow(role = role, navigationStore = navigationStore)
        }
    }
}

@Composable
private fun RoleRow(role: me.lampu.lampcord.shared.model.Role, navigationStore: NavigationStore) {
    val color = if (role.color != 0) Color(role.color) else MaterialTheme.colorScheme.primary
    Surface(
        shape = RoundedCornerShape(8.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Row(
            modifier = Modifier.padding(12.dp).fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(Modifier.size(16.dp).background(color, CircleShape))
            Spacer(Modifier.width(12.dp))
            Text(role.name, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.weight(1f))
            Icon(
                Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

private fun String.managesRoles(): Boolean = toLongOrNull()?.let { (it and (1L shl 28)) != 0L } == true