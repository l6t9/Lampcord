package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.ReadStateStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserGuildSettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.components.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.setClipboardText
import me.lampu.lampcord.shared.utils.getPlatformName
import kotlinx.coroutines.withTimeoutOrNull
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun GuildIcon(
    guild: me.lampu.lampcord.shared.model.Guild,
    isSelected: Boolean,
    onClick: () -> Unit,
    guildStore: GuildStore = koinInject(),
    userGuildSettingsStore: UserGuildSettingsStore = koinInject(),
    readStateStore: ReadStateStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    profileStore: ProfileStore = koinInject(),
    userStore: UserStore = koinInject(),
    arrangeMode: Boolean = false,
    onArrangeMode: () -> Unit = {},
    onDragStart: () -> Unit = {},
    onDrag: (Float) -> Unit = {},
    onDragEnd: () -> Unit = {},
    onDragCancel: () -> Unit = {},
    isDragging: Boolean = false,
    dragOffset: Float = 0f
) {
    val canArrange = getPlatformName() == "android"
    var contextMenuRequest by remember { mutableStateOf(0) }
    val isAnimated = guild.icon?.startsWith("a_") == true
    val iconSizePx = with(LocalDensity.current) { 48.dp.roundToPx() }
    val iconUrl = if (guild.icon != null) {
        val ext = if (isAnimated && isSelected && !me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) "gif" else "png"
        "https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.$ext?size=$iconSizePx"
    } else null

    val userGuildSettings by userGuildSettingsStore.userGuildSettings.collectAsState()
    val readStates by readStateStore.readStates.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()

    val isMuted = remember(guild.id, userGuildSettings[guild.id]) { userGuildSettingsStore.isGuildMuted(guild.id) }
    
    val isUnread = remember(guild.id, readStates, userGuildSettings[guild.id]) { guildStore.isGuildUnread(guild.id) }
    val mentionCount = remember(guild.id, readStates) { guildStore.getGuildMentionCount(guild.id) }

    var showMuteDialog by remember { mutableStateOf(false) }
    var showLeaveDialog by remember { mutableStateOf(false) }

    val errorColor = MaterialTheme.colorScheme.error
    val contextMenuItems = remember(guild, isSelected, settingsStore.userSettings, isMuted, errorColor) {
        val items = mutableListOf(
            ContextMenuItem(if (isMuted) "Unmute Server" else "Mute Server", if (isMuted) Icons.Filled.Notifications else Icons.AutoMirrored.Filled.VolumeOff, onClick = {
                if (isMuted) {
                    guildStore.unmuteGuild(guild.id)
                } else {
                    showMuteDialog = true
                }
            }, group = "Primary"),
            ContextMenuItem("Mark as Read", Icons.Filled.Check, onClick = { guildStore.markGuildAsRead(guild.id) }, group = "Primary"),
            ContextMenuItem("Edit Profile", Icons.Filled.AccountCircle, onClick = {
                navigationStore.navigateToSettings("PROFILES")
            }, group = "Primary")
        )
        if (!isSelected) {
            items.add(ContextMenuItem("Leave Server", Icons.Filled.Logout, onClick = {
                showLeaveDialog = true
            }, color = errorColor, group = "Destructive"))
        }
        if (settingsStore.userSettings?.developer_mode == true) {
            items.add(ContextMenuItem("Copy ID", Icons.Filled.Dns, onClick = { setClipboardText(guild.id) }, group = "Developer"))
        }
        items
    }

    ExpressiveTooltip(
        anchorPosition = TooltipAnchorPosition.End,
        content = tooltipText(guild.name ?: "Server"),
        anchor = {
            ContextMenu(
                items = contextMenuItems
                , enabled = !canArrange
                , openRequest = contextMenuRequest
            ) {
                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .graphicsLayer(clip = false)
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
                ) {
                    RegularGuildItem(
                        isSelected = isSelected,
                        isUnread = isUnread,
                        isMonogram = iconUrl == null,
                        isMuted = isMuted,
                        onClick = onClick,
                        selectedColor = if (iconUrl == null) MaterialTheme.colorScheme.primary else Color.Transparent,
                        unselectedColor = if (iconUrl == null) MaterialTheme.colorScheme.surfaceVariant else Color.Transparent,
                        monogramSelectedColor = if (iconUrl == null) MaterialTheme.colorScheme.onPrimary else Color.Transparent,
                        monogramUnselectedColor = if (iconUrl == null) MaterialTheme.colorScheme.primary else Color.Transparent
                    ) {
                        if (iconUrl != null) {
                            AsyncImage(
                                model = iconUrl,
                                contentDescription = guild.name,
                                modifier = Modifier.fillMaxSize()
                            )
                        } else {
                            val initials = remember(guild.name) {
                                guild.name?.split(" ")?.mapNotNull { it.firstOrNull() }?.joinToString("") ?: "?"
                            }
                            Text(
                                text = initials,
                                style = MaterialTheme.typography.titleMedium,
                                textAlign = TextAlign.Center,
                                maxLines = 1,
                                fontSize = if (initials.length > 3) 12.sp else 16.sp
                            )
                        }
                    }

                    if (mentionCount > 0) {
                        Box(
                            modifier = Modifier
                                .size(48.dp)
                        ) {
                            Surface(
                                color = MaterialTheme.colorScheme.error,
                                shape = CircleShape,
                                modifier = Modifier
                                    .align(Alignment.BottomEnd)
                                    .offset(x = 3.dp, y = 3.dp)
                                    .height(20.dp)
                                    .widthIn(min = 20.dp),
                                shadowElevation = 2.dp,
                                border = if (isSelected) null else androidx.compose.foundation.BorderStroke(2.dp, MaterialTheme.colorScheme.surface)
                            ) {
                                Box(
                                    contentAlignment = Alignment.Center,
                                    modifier = Modifier.padding(horizontal = 5.dp)
                                ) {
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
            }
        }
    )

    if (showMuteDialog) {
        MuteServerDialog(
            guildName = guild.name ?: "Server",
            onDismiss = { showMuteDialog = false },
            onConfirm = { duration ->
                guildStore.muteGuild(guild.id, duration)
                showMuteDialog = false
            }
        )
    }

    if (showLeaveDialog) {
        LeaveServerDialog(
            guildName = guild.name ?: "Server",
            onDismiss = { showLeaveDialog = false },
            onConfirm = {
                guildStore.leaveGuild(guild.id) { if (navigationStore.selectedGuild?.id == guild.id) navigationStore.selectHome() }
                showLeaveDialog = false
            }
        )
    }
}
