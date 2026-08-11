package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role as SemanticsRole
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.model.Role as DiscordRole
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.components.guilds.settings.*
import kotlinx.coroutines.launch

import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator

enum class ServerSettingsSection(val title: String, val icon: ImageVector, val selectedIcon: ImageVector, val category: String = "Settings") {
    OVERVIEW("Overview", Icons.Rounded.Info, Icons.Filled.Info),
    ROLES("Roles", Icons.Rounded.Flag, Icons.Filled.Flag),
    EMOJI("Emoji", Icons.Rounded.Mood, Icons.Filled.Mood),
    STICKERS("Stickers", Icons.Rounded.StickyNote2, Icons.Filled.StickyNote2),
    CHANNELS("Channels", Icons.Rounded.Tag, Icons.Filled.Tag),
    
    AUDIT_LOG("Audit Log", Icons.Rounded.Article, Icons.Filled.Article, "Moderation"),
    
    ENABLE_COMMUNITY("Enable Community", Icons.Rounded.Home, Icons.Filled.Home, "Community"),
    
    MEMBERS("Members", Icons.Rounded.Group, Icons.Filled.Group, "User Management"),
    INVITES("Invites", Icons.Rounded.Link, Icons.Filled.Link, "User Management"),
    BANS("Bans", Icons.Rounded.Block, Icons.Filled.Block, "User Management"),
}

private val sectionPermissions = mapOf(
    ServerSettingsSection.OVERVIEW to listOf(Permission.MANAGE_GUILD),
    ServerSettingsSection.ROLES to listOf(Permission.MANAGE_ROLES),
    ServerSettingsSection.EMOJI to listOf(Permission.MANAGE_GUILD_EXPRESSIONS),
    ServerSettingsSection.STICKERS to listOf(Permission.MANAGE_GUILD_EXPRESSIONS),
    ServerSettingsSection.CHANNELS to listOf(Permission.MANAGE_CHANNELS),
    ServerSettingsSection.AUDIT_LOG to listOf(Permission.VIEW_AUDIT_LOG),
    ServerSettingsSection.MEMBERS to listOf(Permission.MANAGE_GUILD, Permission.KICK_MEMBERS, Permission.BAN_MEMBERS),
    ServerSettingsSection.INVITES to listOf(Permission.MANAGE_GUILD),
    ServerSettingsSection.BANS to listOf(Permission.BAN_MEMBERS)
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ServerSettings(chatState: ChatState, onDismiss: () -> Unit) {
    val guild = chatState.selectedGuild ?: return
    
    val allowedSections = remember(guild.id, chatState.currentMember) {
        ServerSettingsSection.entries.filter { section ->
            val required = sectionPermissions[section] ?: return@filter true
            required.any { chatState.hasPermission(it) }
        }
    }

    if (allowedSections.isEmpty()) {
        LaunchedEffect(Unit) { onDismiss() }
        return
    }

    var selectedCategory by remember { mutableStateOf<ServerSettingsSection?>(null) }
    var selectedRole by remember { mutableStateOf<DiscordRole?>(null) }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp

        if (isCompact) {
            if (selectedRole != null) {
                RoleEditorSubScreen(
                    role = selectedRole!!,
                    guild = guild,
                    chatState = chatState,
                    onBack = { selectedRole = null }
                )
            } else if (selectedCategory == null) {
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

                        val groups = allowedSections.groupBy { it.category }
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
                    ServerSettingsContent(
                        section = selectedCategory!!,
                        guild = guild,
                        chatState = chatState,
                        onRoleClick = { selectedRole = it }
                    )
                }
            }
        } else {
            // Desktop Layout
            androidx.compose.ui.window.Dialog(
                onDismissRequest = onDismiss,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
            ) {
                ServerSettingsDesktopOverlay(
                    chatState = chatState,
                    allowedSections = allowedSections,
                    selectedCategory = selectedCategory,
                    selectedRole = selectedRole,
                    onDismiss = onDismiss,
                    onCategorySelected = { 
                        selectedCategory = it
                        selectedRole = null 
                    },
                    onRoleSelected = { selectedRole = it }
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun ServerSettingsDesktopOverlay(
    chatState: ChatState,
    allowedSections: List<ServerSettingsSection>,
    selectedCategory: ServerSettingsSection?,
    selectedRole: DiscordRole?,
    onDismiss: () -> Unit,
    onCategorySelected: (ServerSettingsSection) -> Unit,
    onRoleSelected: (DiscordRole) -> Unit
) {
    val guild = chatState.selectedGuild ?: return
    val currentSection = selectedCategory ?: allowedSections.firstOrNull() ?: ServerSettingsSection.OVERVIEW

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
                    allowedSections.forEach { section ->
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
                        targetState = Triple(currentSection, selectedRole, guild.id),
                        transitionSpec = {
                            (fadeIn(animationSpec = tween(300)) + slideInVertically(animationSpec = tween(300)) { 20 }).togetherWith(
                                fadeOut(animationSpec = tween(200))
                            )
                        },
                        modifier = Modifier.fillMaxSize(),
                        label = "serverSettingsContent",
                    ) { target ->
                        val section = target.first
                        val role = target.second
                        val scrollState = rememberScrollState()
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .verticalScroll(scrollState)
                                .padding(24.dp),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            if (role != null && section == ServerSettingsSection.ROLES) {
                                RoleEditor(role, guild, chatState)
                            } else {
                                ServerSettingsContent(section, guild, chatState, onRoleClick = onRoleSelected)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ServerSettingsContent(
    section: ServerSettingsSection,
    guild: Guild,
    chatState: ChatState,
    onRoleClick: (DiscordRole) -> Unit
) {
    when (section) {
        ServerSettingsSection.OVERVIEW -> ServerOverview(guild, chatState)
        ServerSettingsSection.ROLES -> ServerRoles(guild, chatState, onRoleClick)
        ServerSettingsSection.EMOJI -> ServerEmoji(guild, chatState)
        ServerSettingsSection.STICKERS -> ServerStickers(guild, chatState)
        ServerSettingsSection.CHANNELS -> ServerChannels(guild, chatState)
        
        ServerSettingsSection.AUDIT_LOG -> ServerAuditLog(guild, chatState)
        
        ServerSettingsSection.ENABLE_COMMUNITY -> Text("Community features coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        
        ServerSettingsSection.MEMBERS -> ServerMembers(guild, chatState)
        ServerSettingsSection.INVITES -> ServerInvites(guild, chatState)
        ServerSettingsSection.BANS -> ServerBans(guild, chatState)
    }
}
