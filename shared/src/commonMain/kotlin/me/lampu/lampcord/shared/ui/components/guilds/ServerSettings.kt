package me.lampu.lampcord.shared.ui.components.guilds

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.server_settings
import org.jetbrains.compose.resources.stringResource

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
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
import me.lampu.lampcord.shared.ui.components.settings.SettingsOverlayFrame
import me.lampu.lampcord.shared.ui.components.settings.SettingsSearchField
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
                            title = { Text(stringResource(Res.string.server_settings)) },
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
            SettingsOverlayFrame(onDismiss = onDismiss) {
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
    var searchQuery by remember { mutableStateOf("") }
    val filteredSections = remember(searchQuery, allowedSections) {
        val query = searchQuery.trim()
        if (query.isBlank()) {
            allowedSections
        } else {
            allowedSections.filter {
                it.title.contains(query, ignoreCase = true) ||
                    it.category.contains(query, ignoreCase = true)
            }
        }
    }

    Row(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .width(360.dp)
                .fillMaxHeight()
                .padding(horizontal = 20.dp, vertical = 16.dp)
        ) {
            SettingsSearchField(
                query = searchQuery,
                onQueryChange = { searchQuery = it },
                onClear = { searchQuery = "" }
            )
            Spacer(Modifier.height(16.dp))
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.spacedBy(4.dp),
                contentPadding = PaddingValues(bottom = 16.dp)
            ) {
                if (searchQuery.isBlank()) {
                    allowedSections.groupBy { it.category }.forEach { (category, sections) ->
                        item(key = "category:$category") {
                            Text(
                                text = category,
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.padding(start = 16.dp, top = 16.dp, bottom = 8.dp)
                            )
                        }
                        itemsIndexed(sections, key = { _, section -> section.name }) { index, section ->
                            ServerSettingsNavItem(
                                section = section,
                                selected = currentSection == section,
                                first = index == 0,
                                last = index == sections.lastIndex,
                                onClick = { onCategorySelected(section) }
                            )
                        }
                    }
                } else if (filteredSections.isEmpty()) {
                    item(key = "no-results") {
                        Text(
                            text = "No settings found",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 24.dp)
                        )
                    }
                } else {
                    itemsIndexed(filteredSections, key = { _, section -> section.name }) { index, section ->
                        ServerSettingsNavItem(
                            section = section,
                            selected = currentSection == section,
                            first = index == 0,
                            last = index == filteredSections.lastIndex,
                            onClick = {
                                searchQuery = ""
                                onCategorySelected(section)
                            }
                        )
                    }
                }
            }
        }

        Column(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
                .padding(end = 12.dp, bottom = 12.dp)
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(80.dp)
                    .padding(start = 32.dp, end = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=80" }
                if (iconUrl != null) {
                    AsyncImage(
                        model = iconUrl,
                        contentDescription = null,
                        modifier = Modifier.size(36.dp).clip(CircleShape)
                    )
                } else {
                    Surface(
                        modifier = Modifier.size(36.dp),
                        shape = CircleShape,
                        color = MaterialTheme.colorScheme.secondaryContainer
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Text(
                                text = guild.name?.take(1) ?: "?",
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSecondaryContainer
                            )
                        }
                    }
                }
                Spacer(Modifier.width(12.dp))
                Text(
                    text = guild.name ?: "Server Settings",
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f)
                )
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Default.Close, "Close", tint = MaterialTheme.colorScheme.onSurface, modifier = Modifier.size(20.dp))
                }
            }

            Surface(
                modifier = Modifier.fillMaxWidth().weight(1f),
                shape = RoundedCornerShape(28.dp),
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

@Composable
private fun ServerSettingsNavItem(
    section: ServerSettingsSection,
    selected: Boolean,
    first: Boolean,
    last: Boolean,
    onClick: () -> Unit
) {
    val background by animateColorAsState(
        targetValue = if (selected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainer,
        label = "serverSettingsNavItem"
    )
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .clip(connectedServerSettingsShape(first, last))
            .background(background)
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(modifier = Modifier.size(32.dp), contentAlignment = Alignment.Center) {
            Icon(
                imageVector = if (selected) section.selectedIcon else section.icon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(24.dp)
            )
        }
        Spacer(Modifier.width(16.dp))
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                text = section.title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = section.category,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

private fun connectedServerSettingsShape(first: Boolean, last: Boolean) = RoundedCornerShape(
    topStart = if (first) 20.dp else 5.dp,
    topEnd = if (first) 20.dp else 5.dp,
    bottomStart = if (last) 20.dp else 5.dp,
    bottomEnd = if (last) 20.dp else 5.dp
)

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
