package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.runtime.*
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.ui.input.pointer.PointerEventPass
import kotlinx.coroutines.withTimeoutOrNull
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.GuildFolder
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.*
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.getPlatformName

private val FolderIconSize = 48.dp
private val PreviewIconSize = 20.dp
private val PreviewIconOffset = 10.dp

@Composable
fun FolderPreviewGrid(
    folder: GuildFolder,
    guildStore: GuildStore = koinInject()
) {
    val guilds by guildStore.guilds.collectAsState()
    val guildsInFolder = folder.guild_ids.mapNotNull { el ->
        val id = el.jsonPrimitive.contentOrNull ?: return@mapNotNull null
        guilds.find { it.id == id }
    }.take(4)

    val d = PreviewIconOffset

    Box(modifier = Modifier.fillMaxSize()) {
        when (guildsInFolder.size) {
            1 -> GridIcon(guildsInFolder[0], 0.dp, 0.dp)
            2 -> {
                GridIcon(guildsInFolder[0], -d, 0.dp)
                GridIcon(guildsInFolder[1], d, 0.dp)
            }
            3 -> {
                GridIcon(guildsInFolder[0], 0.dp, -d)
                GridIcon(guildsInFolder[1], -d, d)
                GridIcon(guildsInFolder[2], d, d)
            }
            4 -> {
                GridIcon(guildsInFolder[0], -d, -d)
                GridIcon(guildsInFolder[1], d, -d)
                GridIcon(guildsInFolder[2], -d, d)
                GridIcon(guildsInFolder[3], d, d)
            }
        }
    }
}

@Composable
private fun BoxScope.GridIcon(guild: Guild, x: Dp, y: Dp) {
    Box(Modifier.align(Alignment.Center).offset(x = x, y = y).size(PreviewIconSize)) {
        PreviewIcon(guild)
    }
}

@Composable
fun PreviewIcon(guild: Guild) {
    val iconUrl = if (guild.icon != null) {
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=48"
    } else null

    Surface(
        modifier = Modifier.fillMaxSize(),
        shape = RoundedCornerShape(6.dp),
        color = MaterialTheme.colorScheme.surfaceVariant,
        shadowElevation = 0.dp,
        tonalElevation = 0.dp
    ) {
        if (iconUrl != null) {
            AsyncImage(
                model = iconUrl,
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                filterQuality = FilterQuality.Low
            )
        } else {
            Box(contentAlignment = Alignment.Center) {
                Text(
                    text = guild.name?.take(1) ?: "?",
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 8.sp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
fun GuildFolderItem(
    folder: GuildFolder,
    navigationStore: NavigationStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    readStateStore: ReadStateStore = koinInject(),
    userGuildSettingsStore: UserGuildSettingsStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    gatewayManager: GatewayManager = koinInject()
    , arrangeMode: Boolean = false
    , onArrangeMode: () -> Unit = {}
    , onDragStart: () -> Unit = {}
    , onDrag: (Float) -> Unit = {}
    , onDragEnd: () -> Unit = {}
    , onDragCancel: () -> Unit = {}
    , isDragging: Boolean = false
    , dragOffset: Float = 0f
) {
    val canArrange = getPlatformName() == "android"
    var contextMenuRequest by remember { mutableStateOf(0) }
    var expanded by remember { mutableStateOf(false) }
    var showFolderSettings by remember { mutableStateOf(false) }
    val folderColor = folder.color?.let { Color(it.toLong() or 0xFF000000L) } ?: MaterialTheme.colorScheme.primary
    
    val guildIds = remember(folder.guild_ids) { folder.guild_ids.mapNotNull { el -> el.jsonPrimitive.contentOrNull } }
    val isAnyChildSelected = guildIds.any { id -> id == navigationStore.selectedGuild?.id }
    val reduceMotion = Settings.shared.reduceMotion
    
    val readStates by readStateStore.readStates.collectAsState()
    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()

    val isUnread by remember(folder, readStates, userGuildSettings) {
        derivedStateOf { guildStore.isFolderUnread(folder) }
    }
    val mentionCount by remember(folder, readStates, userGuildSettings) {
        derivedStateOf { guildStore.getFolderMentionCount(folder) }
    }

    val contextMenuItems = remember(folder, guildIds) {
        listOf(
            ContextMenuItem("Mark as Read", Icons.Filled.Check, onClick = {
                guildStore.markFolderAsRead(folder)
            }, group = "Primary"),
            ContextMenuItem("Folder Settings", Icons.Filled.Settings, onClick = {
                showFolderSettings = true
            }, group = "Primary")
        )
    }

    val surfaceColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
    val expansionProgress by animateFloatAsState(
        targetValue = if (expanded) 1f else 0f,
        animationSpec = if (reduceMotion) snap() else spring()
    )

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .fillMaxWidth()
            .drawBehind {
                if (expansionProgress > 0f) {
                    val wellWidth = FolderIconSize.toPx()
                    val x = (size.width - wellWidth) / 2
                    drawRoundRect(
                        color = surfaceColor,
                        topLeft = Offset(x, 0f),
                        size = Size(wellWidth, size.height * expansionProgress),
                        cornerRadius = CornerRadius(wellWidth / 2) // Semicircle rounding at top and bottom
                    )
                }
            }
    ) {
        // Folder Icon Row
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(FolderIconSize)
                .graphicsLayer(clip = false),
            contentAlignment = Alignment.Center
        ) {
            val showIndicator = (!expanded && isAnyChildSelected) || (isUnread && !expanded)
            if (showIndicator) {
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterStart)
                        .width(4.dp)
                        .height(if (isAnyChildSelected) 38.dp else 8.dp) // Stylized indicator
                        .clip(RoundedCornerShape(topEnd = 4.dp, bottomEnd = 4.dp))
                        .background(MaterialTheme.colorScheme.onSurface)
                )
            }

            val interactionSource = remember { MutableInteractionSource() }
            val isHovered by interactionSource.collectIsHoveredAsState()

            val folderBgColor by animateColorAsState(
                targetValue = when {
                    expanded -> folderColor.copy(alpha = if (isHovered) 0.2f else 0.1f)
                    else -> folderColor.copy(alpha = if (isHovered) 0.35f else 0.2f)
                },
                animationSpec = if (reduceMotion) snap() else spring()
            )

            ExpressiveTooltip(
                anchorPosition = TooltipAnchorPosition.End,
                content = tooltipText(folder.name ?: "Folder"),
                interactionSource = interactionSource,
                anchor = {
                    ContextMenu(items = contextMenuItems, enabled = !canArrange, openRequest = contextMenuRequest) {
                        Box(
                            modifier = Modifier
                                .size(FolderIconSize)
                                .graphicsLayer { translationY = if (isDragging) dragOffset else 0f }
                                .pointerInput(canArrange) {
                                    if (canArrange) {
                                        awaitPointerEventScope {
                                            while (true) {
                                                val down = awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                                                val movedEarly = withTimeoutOrNull<Boolean>(220L) {
                                                    while (true) {
                                                        val change = awaitPointerEvent(PointerEventPass.Initial).changes.firstOrNull { it.id == down.id }
                                                        if (change == null || !change.pressed) return@withTimeoutOrNull true
                                                        if ((change.position - down.position).getDistance() > viewConfiguration.touchSlop) return@withTimeoutOrNull true
                                                    }
                                                    false
                                                } ?: false
                                                if (movedEarly) continue
                                                onDragStart()
                                                val movedDuringGrace = withTimeoutOrNull<Boolean>(280L) {
                                                    while (true) {
                                                        val change = awaitPointerEvent(PointerEventPass.Initial).changes.firstOrNull { it.id == down.id }
                                                        if (change == null || !change.pressed) return@withTimeoutOrNull false
                                                        if ((change.position - down.position).getDistance() > viewConfiguration.touchSlop) return@withTimeoutOrNull true
                                                    }
                                                    false
                                                } ?: false
                                                if (!movedDuringGrace) {
                                                    onDragCancel()
                                                    contextMenuRequest++
                                                    continue
                                                }
                                                while (true) {
                                                    val change = awaitPointerEvent(PointerEventPass.Initial).changes.firstOrNull { it.id == down.id }
                                                    if (change == null || !change.pressed) {
                                                        onDragEnd()
                                                        break
                                                    }
                                                    change.consume()
                                                    onDrag((change.position - change.previousPosition).y)
                                                }
                                            }
                                        }
                                    }
                                }
                                .clickable(
                                    interactionSource = interactionSource,
                                    indication = null,
                                    onClick = { expanded = !expanded }
                                ),
                            contentAlignment = Alignment.Center
                        ) {
                            Box(
                                modifier = Modifier
                                    .matchParentSize()
                                    .clip(CircleShape)
                                    .background(folderBgColor)
                            )
                            if (expanded) {
                                Icon(
                                    imageVector = Icons.Filled.FolderOpen,
                                    contentDescription = folder.name ?: "Folder",
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(28.dp)
                                )
                            } else {
                                FolderPreviewGrid(folder)
                            }
                        }
                    }
                }
            )
            
            if (mentionCount > 0 && !expanded) {
                Box(modifier = Modifier.size(FolderIconSize).graphicsLayer(clip = false)) {
                    Surface(
                        color = MaterialTheme.colorScheme.error,
                        shape = CircleShape,
                        modifier = Modifier
                            .align(Alignment.BottomEnd)
                            .offset(x = 3.dp, y = 3.dp)
                            .height(20.dp)
                            .widthIn(min = 20.dp),
                        shadowElevation = 2.dp,
                        border = androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.surface)
                    ) {
                        Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(horizontal = 5.dp)) {
                            Text(
                                text = if (mentionCount > 99) "99+" else mentionCount.toString(),
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onError,
                                fontWeight = FontWeight.ExtraBold,
                                fontSize = 11.sp,
                                maxLines = 1
                            )
                        }
                    }
                }
            }
        }

        AnimatedVisibility(
            visible = expanded,
            enter = if (reduceMotion) EnterTransition.None else expandVertically(animationSpec = spring(stiffness = 300f)) + fadeIn(),
            exit = if (reduceMotion) ExitTransition.None else shrinkVertically(animationSpec = spring(stiffness = 300f)) + fadeOut()
        ) {
            // Expanded area
            Column(
                modifier = Modifier
                    .fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Spacer(Modifier.height(4.dp)) // Small gap after folder icon
                val guilds by guildStore.guilds.collectAsState()
                folder.guild_ids.forEach { el ->
                    val guildId = el.jsonPrimitive.contentOrNull ?: return@forEach
                    val guild = guilds.find { it.id == guildId }
                    if (guild != null) {
                        GuildIcon(
                            guild = guild,
                            isSelected = navigationStore.selectedGuild?.id == guild.id,
                            onClick = { navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) } }
                        )
                    }
                }
            }
        }
    }

    if (showFolderSettings) {
        FolderSettingsDialog(
            folder = folder,
            onDismiss = { showFolderSettings = false }
        )
    }
}
