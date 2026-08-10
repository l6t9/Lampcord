package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*
import kotlinx.coroutines.launch

enum class ServerSettingsSection(val title: String, val icon: ImageVector, val selectedIcon: ImageVector, val category: String = "Settings") {
    OVERVIEW("Overview", Icons.Rounded.Info, Icons.Filled.Info),
    MODERATION("Moderation", Icons.Rounded.Security, Icons.Filled.Security),
    AUDIT_LOG("Audit Log", Icons.Rounded.Description, Icons.Filled.Description),
    CHANNELS("Channels", Icons.Rounded.Tag, Icons.Filled.Tag),
    INTEGRATIONS("Integrations", Icons.Rounded.SportsEsports, Icons.Filled.SportsEsports),
    SECURITY("Security", Icons.Rounded.Security, Icons.Filled.Security), // Using Security as fallback for Lock
    EMOJI("Emoji", Icons.Rounded.Mood, Icons.Filled.Mood),
    STICKERS("Stickers", Icons.Rounded.StickyNote2, Icons.Filled.StickyNote2),
    
    ENABLE_COMMUNITY("Enable Community", Icons.Rounded.Home, Icons.Filled.Home, "Community Settings"),
    
    MEMBERS("Members", Icons.Rounded.Group, Icons.Filled.Group, "User Management"),
    ROLES("Roles", Icons.Rounded.Flag, Icons.Filled.Flag, "User Management"),
    INVITES("Invites", Icons.Rounded.Link, Icons.Filled.Link, "User Management"),
    BANS("Bans", Icons.Rounded.Block, Icons.Filled.Block, "User Management"),
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerSettings(chatState: ChatState, onDismiss: () -> Unit) {
    val guild = chatState.selectedGuild ?: return
    var selectedCategory by remember { mutableStateOf<ServerSettingsSection?>(null) }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp

        if (isCompact) {
            if (selectedCategory == null) {
                // Mobile List View
                Scaffold(
                    topBar = {
                        TopAppBar(
                            title = { Text("Server Settings") },
                            navigationIcon = {
                                IconButton(onClick = onDismiss) {
                                    Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                                }
                            }
                        )
                    }
                ) { padding ->
                    LazyColumn(
                        modifier = Modifier.fillMaxSize().padding(padding).background(MaterialTheme.colorScheme.surface),
                        contentPadding = PaddingValues(vertical = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        item {
                            Column(
                                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 16.dp),
                                horizontalAlignment = Alignment.CenterHorizontally
                            ) {
                                val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=160" }
                                if (iconUrl != null) {
                                    AsyncImage(
                                        model = iconUrl,
                                        contentDescription = null,
                                        modifier = Modifier.size(100.dp).clip(RoundedCornerShape(20.dp))
                                    )
                                } else {
                                    Box(
                                        modifier = Modifier.size(100.dp).background(MaterialTheme.colorScheme.secondaryContainer, RoundedCornerShape(20.dp)),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Text(guild.name?.take(1) ?: "?", style = MaterialTheme.typography.headlineLarge)
                                    }
                                }
                                Spacer(Modifier.height(12.dp))
                                Text(guild.name ?: "", style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                            }
                        }

                        val groups = ServerSettingsSection.entries.groupBy { it.category }
                        groups.forEach { (category, sections) ->
                            item {
                                Material3SettingsGroup(
                                    title = category,
                                    items = sections.map { section ->
                                        Material3SettingsItem(
                                            icon = section.icon,
                                            title = { Text(section.title) },
                                            onClick = { selectedCategory = section }
                                        )
                                    }
                                )
                            }
                        }
                    }
                }
            } else {
                SettingsSubScreen(
                    title = selectedCategory!!.title,
                    onNavigateBack = { selectedCategory = null },
                ) {
                    ServerSettingsContent(selectedCategory!!, guild, chatState)
                }
            }
        } else {
            // Desktop Layout
            androidx.compose.ui.window.Dialog(
                onDismissRequest = onDismiss,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
            ) {
                ServerSettingsDesktopOverlay(chatState, selectedCategory, onDismiss, { selectedCategory = it })
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun ServerSettingsDesktopOverlay(
    chatState: ChatState,
    selectedCategory: ServerSettingsSection?,
    onDismiss: () -> Unit,
    onCategorySelected: (ServerSettingsSection) -> Unit
) {
    val guild = chatState.selectedGuild ?: return
    val currentSection = selectedCategory ?: ServerSettingsSection.OVERVIEW

    Surface(
        modifier = Modifier
            .widthIn(max = 1200.dp)
            .fillMaxWidth(0.95f)
            .heightIn(max = 850.dp)
            .fillMaxHeight(0.9f)
            .clip(MaterialTheme.shapes.large),
        color = MaterialTheme.colorScheme.surfaceContainerLowest,
        shape = MaterialTheme.shapes.large
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            // Header
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 8.dp, start = 16.dp, end = 16.dp)
            ) {
                Text(
                    text = "Server Settings",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.align(Alignment.Center)
                )
                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier.align(Alignment.CenterEnd)
                ) {
                    Icon(Icons.Default.Close, "Close", tint = MaterialTheme.colorScheme.onSurface, modifier = Modifier.size(20.dp))
                }
            }

            Row(modifier = Modifier.fillMaxSize()) {
                // Wide Navigation Rail
                val railState = rememberWideNavigationRailState(initialValue = WideNavigationRailValue.Expanded)
                WideNavigationRail(
                    state = railState,
                    colors = WideNavigationRailDefaults.colors(
                        containerColor = MaterialTheme.colorScheme.surfaceContainerLowest
                    ),
                    windowInsets = WindowInsets(0.dp),
                    contentPadding = PaddingValues(0.dp),
                    header = {
                        val scope = rememberCoroutineScope()
                        val isExpanded = railState.currentValue == WideNavigationRailValue.Expanded
                        Column(
                            horizontalAlignment = Alignment.Start,
                            modifier = Modifier.padding(start = 20.dp, top = 12.dp, bottom = 4.dp)
                        ) {
                            IconButton(onClick = { scope.launch { railState.toggle() } }) {
                                Icon(
                                    imageVector = if (isExpanded) Icons.AutoMirrored.Filled.MenuOpen else Icons.Filled.Menu,
                                    contentDescription = "Toggle Sidebar",
                                )
                            }

                            ExtendedFloatingActionButton(
                                onClick = { /* Placeholder */ },
                                expanded = isExpanded,
                                icon = {
                                    val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=64" }
                                    if (iconUrl != null) {
                                        AsyncImage(
                                            model = iconUrl,
                                            contentDescription = null,
                                            modifier = Modifier.size(24.dp).clip(CircleShape)
                                        )
                                    } else {
                                        Icon(Icons.Filled.Settings, null)
                                    }
                                },
                                text = { Text(guild.name ?: "Server") },
                                containerColor = MaterialTheme.colorScheme.primaryContainer,
                                contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                                shape = RoundedCornerShape(16.dp),
                                modifier = Modifier.padding(top = 8.dp)
                            )
                        }
                    }
                ) {
                    val railSections = listOf(
                        ServerSettingsSection.OVERVIEW,
                        ServerSettingsSection.ROLES,
                        ServerSettingsSection.EMOJI,
                        ServerSettingsSection.MEMBERS,
                        ServerSettingsSection.INVITES,
                        ServerSettingsSection.BANS
                    )

                    railSections.forEach { section ->
                        val isSelected = currentSection == section
                        
                        WideNavigationRailItem(
                            selected = isSelected,
                            railExpanded = railState.currentValue == WideNavigationRailValue.Expanded,
                            onClick = { onCategorySelected(section) },
                            icon = {
                                Icon(
                                    if (isSelected) section.selectedIcon else section.icon,
                                    contentDescription = section.title,
                                )
                            },
                            label = {
                                Text(
                                    section.title,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        )
                    }
                }

                // Main Content Area
                Surface(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight()
                        .padding(end = 8.dp, bottom = 8.dp),
                    shape = MaterialTheme.shapes.medium,
                    color = MaterialTheme.colorScheme.surfaceContainerLow
                ) {
                    AnimatedContent(
                        targetState = currentSection,
                        transitionSpec = {
                            (fadeIn(animationSpec = tween(300)) + slideInVertically(animationSpec = tween(300)) { 20 }).togetherWith(
                                fadeOut(animationSpec = tween(200))
                            )
                        },
                        modifier = Modifier.fillMaxSize(),
                        label = "serverSettingsContent",
                    ) { section ->
                        val scrollState = rememberScrollState()
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .verticalScroll(scrollState)
                                .padding(24.dp),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            ServerSettingsContent(section, guild, chatState)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ServerSettingsContent(section: ServerSettingsSection, guild: Guild, chatState: ChatState) {
    when (section) {
        ServerSettingsSection.OVERVIEW -> ServerOverview(guild, chatState)
        ServerSettingsSection.ROLES -> ServerRoles(guild, chatState)
        ServerSettingsSection.EMOJI -> ServerEmoji(guild, chatState)
        ServerSettingsSection.STICKERS -> ServerStickers(guild, chatState)
        ServerSettingsSection.MEMBERS -> ServerMembers(guild, chatState)
        
        ServerSettingsSection.MODERATION -> Text("Moderation settings coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.AUDIT_LOG -> Text("Audit Log coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.CHANNELS -> Text("Channel management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.INTEGRATIONS -> Text("Integrations coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.SECURITY -> Text("Security settings coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.ENABLE_COMMUNITY -> Text("Community features coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.INVITES -> Text("Invite management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        ServerSettingsSection.BANS -> Text("Ban management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun ServerOverview(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopServerOverview(guild, chatState)
    } else {
        MobileServerOverview(guild, chatState)
    }
}

@Composable
private fun DesktopServerOverview(guild: Guild, chatState: ChatState) {
    DesktopSettingsLayout {
        DesktopSettingsSection(
            title = "Server Details",
            icon = Icons.Filled.Info
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(20.dp)) {
                Column {
                    Text("Server Name", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
                    Text(guild.name ?: "Unnamed Server", style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurface)
                }
                Column {
                    Text("Server ID", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
                    Text(guild.id, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurface)
                }
            }
        }

        DesktopSettingsSection(
            title = "Security",
            icon = Icons.Filled.Security
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(24.dp)) {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Verification Level", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    DesktopButtonGroupSelection(
                        options = listOf(0, 1, 2, 3, 4),
                        selectedOption = guild.verification_level ?: 0,
                        onOptionSelected = { /* TODO */ },
                        iconProvider = { level: Int, isSelected ->
                            when (level) {
                                0 -> if (isSelected) Icons.Filled.Block else Icons.Rounded.Block
                                1 -> if (isSelected) Icons.Filled.ChevronRight else Icons.Rounded.ChevronRight
                                2 -> if (isSelected) Icons.Filled.BarChart else Icons.Rounded.BarChart
                                else -> if (isSelected) Icons.Filled.Security else Icons.Rounded.Security
                            }
                        },
                        labelProvider = {
                            when (it) {
                                0 -> "None"
                                1 -> "Low"
                                2 -> "Medium"
                                3 -> "High"
                                else -> "Highest"
                            }
                        }
                    )
                }

                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("Explicit Content Filter", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    DesktopButtonGroupSelection(
                        options = listOf(0, 1, 2),
                        selectedOption = guild.explicit_content_filter ?: 0,
                        onOptionSelected = { /* TODO */ },
                        iconProvider = { filter: Int, isSelected ->
                            when (filter) {
                                0 -> if (isSelected) Icons.Filled.Block else Icons.Rounded.Block
                                1 -> if (isSelected) Icons.Filled.Person else Icons.Rounded.Person
                                else -> if (isSelected) Icons.Filled.Groups else Icons.Rounded.Groups
                            }
                        },
                        labelProvider = {
                            when (it) {
                                0 -> "Don't scan"
                                1 -> "Members"
                                else -> "Everyone"
                            }
                        }
                    )
                }
            }
        }

        DesktopSettingsSection(
            title = "System Messages",
            icon = Icons.Filled.Chat
        ) {
            Column {
                Text("System Messages Channel", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
                Text(
                    chatState.channels.find { it.id == guild.system_channel_id }?.name ?: "No system messages channel",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface
                )
            }
        }
    }
}

@Composable
private fun MobileServerOverview(guild: Guild, chatState: ChatState) {
    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Material3SettingsGroup(
            title = "Server Details",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Server Name") },
                    description = { Text(guild.name ?: "") },
                    onClick = { /* TODO: edit name */ }
                ),
                Material3SettingsItem(
                    title = { Text("Server ID") },
                    description = { Text(guild.id) },
                    onClick = { /* TODO: copy ID */ }
                )
            )
        )

        Material3SettingsGroup(
            title = "Security",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Verification Level") },
                    description = {
                        Text(
                            when (guild.verification_level) {
                                0 -> "None"
                                1 -> "Low"
                                2 -> "Medium"
                                3 -> "High"
                                4 -> "Highest"
                                else -> "None"
                            }
                        )
                    },
                    onClick = { /* TODO: update verification level */ }
                ),
                Material3SettingsItem(
                    title = { Text("Explicit Content Filter") },
                    description = {
                        Text(
                            when (guild.explicit_content_filter) {
                                0 -> "Don't scan any messages"
                                1 -> "Scan messages from members without a role"
                                2 -> "Scan messages from all members"
                                else -> "Don't scan"
                            }
                        )
                    },
                    onClick = { /* TODO: update filter */ }
                )
            )
        )

        Material3SettingsGroup(
            title = "System Messages",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("System Messages Channel") },
                    description = {
                        Text(
                            chatState.channels.find { it.id == guild.system_channel_id }?.name
                                ?: "No system messages channel"
                        )
                    },
                    onClick = { /* TODO */ }
                )
            )
        )
    }
}

@Composable
private fun ServerRoles(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }

    if (!isMobile) {
        DesktopServerRoles(guild, chatState)
    } else {
        MobileServerRoles(guild, chatState)
    }
}

@Composable
private fun DesktopServerRoles(guild: Guild, chatState: ChatState) {
    DesktopSettingsLayout {
        DesktopSettingsSection(
            title = "Roles",
            icon = Icons.Filled.Flag
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                guild.roles.sortedByDescending { it.position }.forEach { role ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(48.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.15f))
                            .clickable { /* TODO: Edit Role */ }
                            .padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(16.dp)
                                .background(
                                    if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurfaceVariant,
                                    CircleShape
                                )
                        )
                        Text(role.name, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.weight(1f))
                        if (role.managed) {
                            Icon(
                                Icons.Filled.Security,
                                null,
                                modifier = Modifier.size(16.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun MobileServerRoles(guild: Guild, chatState: ChatState) {
    Material3SettingsGroup(
        items = guild.roles.sortedByDescending { it.position }.map { role ->
            Material3SettingsItem(
                leadingContent = {
                    Box(
                        modifier = Modifier
                            .size(12.dp)
                            .background(
                                if (role.color != 0) Color(role.color.toLong() or 0xFF000000L) else MaterialTheme.colorScheme.onSurfaceVariant,
                                CircleShape
                            )
                    )
                },
                title = { Text(role.name) },
                trailingContent = {
                    if (role.managed) {
                        Icon(
                            Icons.Filled.Security,
                            null,
                            modifier = Modifier.size(16.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                },
                onClick = { /* TODO: Edit Role */ }
            )
        }
    )
}

@Composable
private fun ServerEmoji(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }
    
    if (isMobile) {
        if (guild.emojis.isEmpty()) {
            Text("No custom emojis", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
        } else {
            FlowRow(
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                guild.emojis.forEach { emoji ->
                    val url = "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=96"
                    me.lampu.lampcord.shared.ui.components.AsyncImage(
                        model = url,
                        contentDescription = emoji.name,
                        modifier = Modifier.size(48.dp)
                    )
                }
            }
        }
    } else {
        DesktopSettingsLayout {
            DesktopSettingsSection(title = "Emoji", icon = Icons.Filled.Mood) {
                if (guild.emojis.isEmpty()) {
                    Text("No custom emojis", color = MaterialTheme.colorScheme.onSurfaceVariant)
                } else {
                    FlowRow(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        guild.emojis.forEach { emoji ->
                            val url = "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=96"
                            me.lampu.lampcord.shared.ui.components.AsyncImage(
                                model = url,
                                contentDescription = emoji.name,
                                modifier = Modifier.size(64.dp)
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ServerStickers(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }
    if (isMobile) {
        Text("Sticker management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
    } else {
        DesktopSettingsLayout {
            DesktopSettingsSection(title = "Stickers", icon = Icons.Filled.StickyNote2) {
                Text("Sticker management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
private fun ServerMembers(guild: Guild, chatState: ChatState) {
    val isMobile = me.lampu.lampcord.shared.utils.getPlatformName().let { it == "android" || it == "ios" }
    if (isMobile) {
        Text("Member management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(16.dp))
    } else {
        DesktopSettingsLayout {
            DesktopSettingsSection(title = "Members", icon = Icons.Filled.Group) {
                Text("Member management coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}
