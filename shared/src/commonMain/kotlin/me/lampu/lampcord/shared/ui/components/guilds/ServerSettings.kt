package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.WideNavigationRail
import androidx.compose.material3.WideNavigationRailDefaults
import androidx.compose.material3.WideNavigationRailItem
import androidx.compose.material3.WideNavigationRailValue
import androidx.compose.material3.rememberWideNavigationRailState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.components.guilds.settings.RoleEditor
import me.lampu.lampcord.shared.ui.components.guilds.settings.RoleEditorSubScreen
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerAuditLog
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerBans
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerChannels
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerEmoji
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerInvites
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerMembers
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerOverview
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerRoles
import me.lampu.lampcord.shared.ui.components.guilds.settings.ServerStickers
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.model.Role as DiscordRole

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
fun ServerSettings(
    onDismiss: () -> Unit,
    userStore: UserStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val guild = navigationStore.selectedGuild ?: return
    val currentUser by userStore.currentUser.collectAsState()
    val member = remember(guild.id, currentUser) {
        currentUser?.id?.let { userStore.getMember(guild.id, it) }
    }
    
    val allowedSections = remember(guild.id, member) {
        val currentUserVal = currentUser
        ServerSettingsSection.entries.filter { section ->
            val required = sectionPermissions[section] ?: return@filter true
            required.any { permission ->
                if (currentUserVal == null) false
                else PermissionHelper.hasPermission(member ?: Member(user = currentUserVal), guild, null, permission, currentUserVal.id)
            }
        }
    }

    if (allowedSections.isEmpty()) {
        LaunchedEffect(Unit) { onDismiss() }
        return
    }

    var selectedCategory by remember { mutableStateOf<ServerSettingsSection?>(null) }
    var selectedRole by remember { mutableStateOf<DiscordRole?>(null) }

    PlatformBackHandler(enabled = selectedRole != null || selectedCategory != null) {
        if (selectedRole != null) {
            selectedRole = null
        } else {
            selectedCategory = null
        }
    }

    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val isCompact = maxWidth < 600.dp

        if (isCompact) {
            if (selectedRole != null) {
                RoleEditorSubScreen(
                    role = selectedRole!!,
                    guild = guild,
                    onBack = { selectedRole = null }
                )
            } else if (selectedCategory == null) {
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
                        onRoleClick = { selectedRole = it }
                    )
                }
            }
        } else {
            androidx.compose.ui.window.Dialog(
                onDismissRequest = onDismiss,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
            ) {
                ServerSettingsDesktopOverlay(
                    navigationStore = navigationStore,
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
    navigationStore: NavigationStore,
    allowedSections: List<ServerSettingsSection>,
    selectedCategory: ServerSettingsSection?,
    selectedRole: DiscordRole?,
    onDismiss: () -> Unit,
    onCategorySelected: (ServerSettingsSection) -> Unit,
    onRoleSelected: (DiscordRole) -> Unit
) {
    val guild = navigationStore.selectedGuild ?: return
    val currentSection = selectedCategory ?: allowedSections.firstOrNull() ?: ServerSettingsSection.OVERVIEW
    val reduceMotion = Settings.shared.reduceMotion

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
                            if (reduceMotion) {
                                EnterTransition.None togetherWith ExitTransition.None
                            } else {
                                (fadeIn(animationSpec = tween(300)) + slideInVertically(animationSpec = tween(300)) { 20 }).togetherWith(
                                    fadeOut(animationSpec = tween(200))
                                )
                            }
                        },
                        modifier = Modifier.fillMaxSize(),
                        label = "serverSettingsContent",
                    ) { target ->
                        val (targetSection, targetRole, _) = target
                        val scrollState = rememberScrollState()
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .verticalScroll(scrollState)
                                .padding(24.dp),
                            verticalArrangement = Arrangement.spacedBy(16.dp)
                        ) {
                            if (targetRole != null && targetSection == ServerSettingsSection.ROLES) {
                                RoleEditor(targetRole, guild)
                            } else {
                                ServerSettingsContent(targetSection, guild, onRoleClick = onRoleSelected)
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
    onRoleClick: (DiscordRole) -> Unit
) {
    when (section) {
        ServerSettingsSection.OVERVIEW -> ServerOverview(guild)
        ServerSettingsSection.ROLES -> ServerRoles(guild, onRoleClick)
        ServerSettingsSection.EMOJI -> ServerEmoji(guild)
        ServerSettingsSection.STICKERS -> ServerStickers(guild)
        ServerSettingsSection.CHANNELS -> ServerChannels(guild)
        
        ServerSettingsSection.AUDIT_LOG -> ServerAuditLog(guild)
        
        ServerSettingsSection.ENABLE_COMMUNITY -> Text("Community features coming soon", color = MaterialTheme.colorScheme.onSurfaceVariant)
        
        ServerSettingsSection.MEMBERS -> ServerMembers(guild)
        ServerSettingsSection.INVITES -> ServerInvites(guild)
        ServerSettingsSection.BANS -> ServerBans(guild)
    }
}
