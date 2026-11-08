package me.lampu.lampcord.shared.ui.components

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.form_label_muted
import me.lampu.lampcord.shared.generated.resources.go_live_tile_screen
import me.lampu.lampcord.shared.generated.resources.go_live_watching_user
import me.lampu.lampcord.shared.generated.resources.live
import me.lampu.lampcord.shared.generated.resources.stop_watching
import me.lampu.lampcord.shared.generated.resources.stream_capture_paused
import me.lampu.lampcord.shared.generated.resources.voice_channel_deafened
import me.lampu.lampcord.shared.generated.resources.watch_stream
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.i18n.localized
import org.jetbrains.compose.resources.stringResource

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items as rowItems
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.gateway.VoicePhase
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.VoiceState
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.voice.rememberVoiceJoin
import org.koin.compose.koinInject
import kotlin.math.ceil

private val SpeakingColor = Color(0xFF23A559)
private val LiveColor = Color(0xFFDA373C)
private val LabelScrim = Color.Black.copy(alpha = 0.55f)

/** A participant's camera tile, or the tile standing in for their Go Live stream. */
private data class CallTile(val state: VoiceState, val screen: Boolean) {
    val key get() = state.user_id + if (screen) ":screen" else ""
}

/**
 * The call stage: participant tiles, a focused Go Live stream and the call controls.
 * [onCollapse] and [onToggleExpand] are offered when the stage is embedded above a chat.
 */
@Composable
fun VoiceArea(
    channel: Channel,
    modifier: Modifier = Modifier,
    expanded: Boolean = false,
    onToggleExpand: (() -> Unit)? = null,
    onCollapse: (() -> Unit)? = null,
    voiceStore: VoiceStore = koinInject(),
    userStore: UserStore = koinInject()
) {
    val participants = voiceStore.voiceStates[channel.guild_id ?: "@me"]?.values?.filter { it.channel_id == channel.id }.orEmpty()
    val isJoined = voiceStore.activeChannel?.id == channel.id
    val watched = voiceStore.watchedStream?.takeIf { isJoined }
    val videoFrames by voiceStore.videoFrames.collectAsState()

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.surfaceContainerLowest)
    ) {
        CallStageHeader(channel, participants.size, isJoined, watched, voiceStore, userStore)
        Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
            when {
                participants.isEmpty() -> VoiceEmptyState(channel, isJoined)
                watched != null -> WatchingLayout(channel, watched, participants, videoFrames, voiceStore, userStore)
                else -> TileGrid(participants, videoFrames, isJoined, voiceStore, userStore)
            }
        }
        if (isJoined) {
            CallControls(voiceStore, expanded, onToggleExpand, onCollapse)
        } else {
            VoiceJoinBar(channel, voiceStore, onCollapse)
        }
    }
}

/**
 * Shows the call stage above a DM's chat while a call is happening there. It can be collapsed
 * to a slim bar or expanded to give the call most of the height.
 */
@Composable
fun ColumnScope.DmCallStage(channel: Channel, voiceStore: VoiceStore = koinInject()) {
    if (channel.guild_id != null || channel.type !in listOf(1, 3)) return
    val joined = voiceStore.activeChannel?.id == channel.id
    val ongoing = voiceStore.voiceStates["@me"]?.values?.any { it.channel_id == channel.id } == true
    if (!joined && !ongoing) return
    var collapsed by remember(channel.id) { mutableStateOf(false) }
    var expanded by remember(channel.id) { mutableStateOf(false) }
    if (collapsed) {
        Surface(color = MaterialTheme.colorScheme.surfaceContainerHigh, modifier = Modifier.fillMaxWidth()) {
            Row(
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Icon(Icons.Filled.Call, contentDescription = null, tint = SpeakingColor, modifier = Modifier.size(18.dp))
                Text(
                    if (joined) voiceStatusText(voiceStore) else "Call in progress",
                    style = MaterialTheme.typography.labelLarge,
                    modifier = Modifier.weight(1f)
                )
                TextButton(onClick = { collapsed = false }) { Text("Show call") }
            }
        }
    } else {
        VoiceArea(
            channel,
            modifier = Modifier.fillMaxHeight(if (expanded) 0.85f else 0.55f),
            expanded = expanded,
            onToggleExpand = { expanded = !expanded },
            onCollapse = { collapsed = true }
        )
    }
}

@Composable
private fun CallStageHeader(
    channel: Channel,
    count: Int,
    isJoined: Boolean,
    watched: String?,
    voiceStore: VoiceStore,
    userStore: UserStore
) {
    Row(
        modifier = Modifier.fillMaxWidth().height(52.dp).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        if (watched != null) {
            val streamer = watched.substringAfterLast(':')
            val frame by voiceStore.streamFrame.collectAsState()
            Surface(shape = CircleShape, color = MaterialTheme.colorScheme.surfaceContainerHigh, modifier = Modifier.weight(1f, fill = false)) {
                Row(
                    modifier = Modifier.padding(start = 4.dp, end = 12.dp, top = 4.dp, bottom = 4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    ParticipantAvatar(userStore, channel.guild_id, streamer, 24.dp)
                    Text(
                        localized(Res.string.go_live_watching_user, "username" to participantName(userStore, channel.guild_id, streamer)),
                        style = MaterialTheme.typography.labelLarge,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
            Spacer(Modifier.weight(1f))
            frame?.let {
                HeaderChip("${it.height}p", MaterialTheme.colorScheme.surfaceContainerHigh, MaterialTheme.colorScheme.onSurface)
            }
            LiveBadge()
            FilledTonalButton(
                onClick = voiceStore::stopWatchingStream,
                contentPadding = PaddingValues(horizontal = 12.dp),
                modifier = Modifier.height(36.dp)
            ) {
                Icon(Icons.Filled.Close, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(6.dp))
                Text(stringResource(Res.string.stop_watching), maxLines = 1)
            }
        } else {
            if (isJoined && voiceStore.connection.phase == VoicePhase.SECURE) {
                Surface(shape = RoundedCornerShape(8.dp), color = MaterialTheme.colorScheme.secondaryContainer) {
                    Row(
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        Icon(Icons.Filled.Lock, contentDescription = null, modifier = Modifier.size(14.dp), tint = MaterialTheme.colorScheme.onSecondaryContainer)
                        Text("DAVE encrypted", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.onSecondaryContainer)
                    }
                }
            }
            val duration = voiceStore.voiceConnectionDuration
            Text(
                if (isJoined && voiceStore.isVoiceConnected) "$count in call · ${duration / 60}:${(duration % 60).toString().padStart(2, '0')}"
                else if (isJoined) "Connecting" else "$count in call",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun HeaderChip(text: String, container: Color, content: Color) {
    Text(
        text,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.SemiBold,
        color = content,
        maxLines = 1,
        modifier = Modifier.background(container, RoundedCornerShape(8.dp)).padding(horizontal = 10.dp, vertical = 4.dp)
    )
}

@Composable
private fun LiveBadge() {
    HeaderChip(stringResource(Res.string.live).uppercase(), LiveColor, Color.White)
}

@Composable
private fun WatchingLayout(
    channel: Channel,
    watched: String,
    participants: List<VoiceState>,
    videoFrames: Map<String, ImageBitmap>,
    voiceStore: VoiceStore,
    userStore: UserStore
) {
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val tiles = participants.map { CallTile(it, screen = false) }
        if (maxWidth >= 560.dp) {
            Row(Modifier.fillMaxSize().padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                StreamView(channel, watched, voiceStore, userStore, Modifier.weight(1f).fillMaxHeight())
                Column(
                    modifier = Modifier.width(176.dp).fillMaxHeight().verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(10.dp, Alignment.CenterVertically)
                ) {
                    tiles.forEach { tile ->
                        key(tile.key) {
                            ParticipantTile(tile, videoFrames[tile.state.user_id], true, voiceStore, userStore, Modifier.fillMaxWidth(), compact = true)
                        }
                    }
                }
            }
        } else {
            Column(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                StreamView(channel, watched, voiceStore, userStore, Modifier.weight(1f).fillMaxWidth().padding(horizontal = 12.dp))
                LazyRow(
                    contentPadding = PaddingValues(horizontal = 12.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    rowItems(tiles, key = { it.key }) { tile ->
                        ParticipantTile(tile, videoFrames[tile.state.user_id], true, voiceStore, userStore, Modifier.width(160.dp), compact = true)
                    }
                }
            }
        }
    }
}

@Composable
private fun StreamView(
    channel: Channel,
    watched: String,
    voiceStore: VoiceStore,
    userStore: UserStore,
    modifier: Modifier = Modifier
) {
    val streamer = watched.substringAfterLast(':')
    val frame by voiceStore.streamFrame.collectAsState()
    val label = localized(Res.string.go_live_tile_screen, "username" to participantName(userStore, channel.guild_id, streamer)).replace("!!", "")
    Box(modifier, contentAlignment = Alignment.Center) {
        val image = frame
        if (image != null) {
            Box(
                modifier = Modifier
                    .aspectRatio(image.width.toFloat() / image.height.coerceAtLeast(1), matchHeightConstraintsFirst = true)
                    .border(2.dp, MaterialTheme.colorScheme.primary, RoundedCornerShape(12.dp))
                    .padding(2.dp)
                    .clip(RoundedCornerShape(10.dp))
            ) {
                Image(image, contentDescription = label, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize())
                TileLabel(label, Modifier.align(Alignment.BottomStart).padding(10.dp))
            }
        } else if (voiceStore.streamPaused) {
            Text(stringResource(Res.string.stream_capture_paused), style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            ContainedLoadingIndicator(modifier = Modifier.size(48.dp))
        }
    }
}

/** Lays tiles out at the largest 16:9 size that fits without scrolling, falling back to a scrolling grid. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TileGrid(
    participants: List<VoiceState>,
    videoFrames: Map<String, ImageBitmap>,
    isJoined: Boolean,
    voiceStore: VoiceStore,
    userStore: UserStore
) {
    val tiles = participants.flatMap { state ->
        listOfNotNull(CallTile(state, screen = false), CallTile(state, screen = true).takeIf { state.self_stream == true })
    }
    BoxWithConstraints(Modifier.fillMaxSize().padding(horizontal = 16.dp, vertical = 8.dp), contentAlignment = Alignment.Center) {
        val gap = 12.dp
        val (columns, width) = remember(tiles.size, maxWidth, maxHeight) { fitTiles(tiles.size, maxWidth, maxHeight, gap) }
        if (width < 160.dp) {
            LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 200.dp),
                horizontalArrangement = Arrangement.spacedBy(gap),
                verticalArrangement = Arrangement.spacedBy(gap),
                modifier = Modifier.fillMaxSize()
            ) {
                items(tiles, key = { it.key }) { tile ->
                    ParticipantTile(tile, videoFrames[tile.state.user_id], isJoined, voiceStore, userStore, compact = true)
                }
            }
        } else {
            FlowRow(
                maxItemsInEachRow = columns,
                horizontalArrangement = Arrangement.spacedBy(gap, Alignment.CenterHorizontally),
                verticalArrangement = Arrangement.spacedBy(gap, Alignment.CenterVertically)
            ) {
                tiles.forEach { tile ->
                    key(tile.key) {
                        ParticipantTile(tile, videoFrames[tile.state.user_id], isJoined, voiceStore, userStore, Modifier.width(width), compact = width < 280.dp)
                    }
                }
            }
        }
    }
}

private fun fitTiles(count: Int, width: Dp, height: Dp, gap: Dp): Pair<Int, Dp> {
    var best = 1 to 0.dp
    for (columns in 1..count.coerceAtLeast(1)) {
        val rows = ceil(count / columns.toFloat()).toInt()
        val byWidth = (width - gap * (columns - 1)) / columns
        val byHeight = (height - gap * (rows - 1)) / rows * (16f / 9f)
        val size = minOf(byWidth, byHeight)
        if (size > best.second) best = columns to size
    }
    return best
}

@Composable
private fun ParticipantTile(
    tile: CallTile,
    video: ImageBitmap?,
    isJoined: Boolean,
    voiceStore: VoiceStore,
    userStore: UserStore,
    modifier: Modifier = Modifier,
    compact: Boolean = false
) {
    val state = tile.state
    val name = participantName(userStore, state.guild_id, state.user_id)
    val isSpeaking = !tile.screen && state.user_id in voiceStore.connection.speaking
    val isStreaming = state.self_stream == true
    val isWatched = voiceStore.watchedStream?.endsWith(":${state.user_id}") == true
    val canWatch = tile.screen && isStreaming && isJoined && state.user_id != userStore.currentUser.value?.id && !isWatched
    val border by animateColorAsState(if (isSpeaking) SpeakingColor else Color.Transparent)
    val shape = RoundedCornerShape(if (compact) 12.dp else 16.dp)

    Surface(
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shape = shape,
        border = if (isSpeaking) BorderStroke(3.dp, border) else null,
        modifier = modifier.aspectRatio(16f / 9f)
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            if (!tile.screen && video != null && state.self_video) {
                Image(video, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
            } else {
                Column(
                    modifier = Modifier.align(Alignment.Center),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    ParticipantAvatar(userStore, state.guild_id, state.user_id, if (compact) 48.dp else 80.dp)
                    if (canWatch) {
                        Button(onClick = { voiceStore.watchStream(state.user_id) }, modifier = Modifier.height(if (compact) 32.dp else 40.dp)) {
                            Icon(Icons.Filled.Visibility, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Text(stringResource(Res.string.watch_stream), maxLines = 1)
                        }
                    }
                }
            }

            if (tile.screen || (compact && isStreaming)) {
                Box(Modifier.align(if (tile.screen) Alignment.TopStart else Alignment.TopEnd).padding(8.dp)) { LiveBadge() }
            }

            TileLabel(
                if (tile.screen) localized(Res.string.go_live_tile_screen, "username" to name).replace("!!", "") else name,
                Modifier.align(Alignment.BottomStart).padding(8.dp)
            )

            if (!tile.screen && (state.self_deaf || state.deaf || state.self_mute || state.mute)) {
                val deafened = state.self_deaf || state.deaf
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(8.dp)
                        .background(LabelScrim, RoundedCornerShape(8.dp))
                        .padding(5.dp)
                ) {
                    Icon(
                        if (deafened) Icons.Filled.HeadsetOff else Icons.Filled.MicOff,
                        contentDescription = stringResource(if (deafened) Res.string.voice_channel_deafened else Res.string.form_label_muted),
                        modifier = Modifier.size(16.dp),
                        tint = MaterialTheme.colorScheme.error
                    )
                }
            }
        }
    }
}

@Composable
private fun TileLabel(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.SemiBold,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        color = Color(0xFFF2F3F5),
        modifier = modifier
            .background(LabelScrim, RoundedCornerShape(8.dp))
            .padding(horizontal = 8.dp, vertical = 3.dp)
    )
}

@Composable
private fun ParticipantAvatar(userStore: UserStore, guildId: String?, userId: String, size: Dp) {
    val user = userStore.getUser(userId)
    val member = guildId?.let { userStore.getMember(it, userId) }
    Box(Modifier.size(size)) {
        AvatarWithDecoration(
            avatarUrl = user?.avatar?.let { CdnUrls.getUserAvatarUrl(userId, it, 160) },
            decorationData = member?.avatar_decoration_data ?: user?.avatar_decoration_data,
            size = size
        )
    }
}

private fun participantName(userStore: UserStore, guildId: String?, userId: String): String {
    val user = userStore.getUser(userId)
    val member = guildId?.let { userStore.getMember(it, userId) }
    return member?.nick ?: user?.global_name ?: user?.username ?: "Unknown User"
}

@Composable
private fun CallControls(
    voiceStore: VoiceStore,
    expanded: Boolean,
    onToggleExpand: (() -> Unit)?,
    onCollapse: (() -> Unit)?
) {
    BoxWithConstraints(Modifier.fillMaxWidth().padding(vertical = 12.dp)) {
        val narrow = maxWidth < 480.dp
        if (onCollapse != null) {
            IconButton(onClick = onCollapse, modifier = Modifier.align(Alignment.CenterStart).padding(start = 8.dp)) {
                Icon(Icons.Filled.ExpandMore, "Collapse call", tint = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        Row(
            modifier = Modifier.align(Alignment.Center),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Surface(shape = RoundedCornerShape(28.dp), color = MaterialTheme.colorScheme.surfaceContainerHighest) {
                Row(Modifier.padding(8.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    ControlButton(
                        if (voiceStore.selfMuted) Icons.Filled.MicOff else Icons.Filled.Mic,
                        if (voiceStore.selfMuted) "Unmute" else "Mute",
                        active = voiceStore.selfMuted,
                        onClick = voiceStore::toggleVoiceMute
                    )
                    ControlButton(
                        if (voiceStore.selfDeafened) Icons.Filled.HeadsetOff else Icons.Filled.Headphones,
                        if (voiceStore.selfDeafened) "Undeafen" else "Deafen",
                        active = voiceStore.selfDeafened,
                        onClick = voiceStore::toggleVoiceDeaf
                    )
                    if (!narrow) {
                        // Sending camera and screen video is not implemented yet; only receiving is.
                        ControlButton(Icons.Filled.Videocam, "Turn on camera", enabled = false) {}
                        ControlButton(Icons.Filled.ScreenShare, "Share your screen", enabled = false) {}
                    }
                }
            }
            Surface(
                onClick = { voiceStore.disconnectFromVoice() },
                shape = RoundedCornerShape(28.dp),
                color = MaterialTheme.colorScheme.error,
                contentColor = MaterialTheme.colorScheme.onError,
                modifier = Modifier.size(width = 72.dp, height = 56.dp)
            ) {
                Box(contentAlignment = Alignment.Center) { Icon(Icons.Filled.CallEnd, "Disconnect") }
            }
        }
        if (onToggleExpand != null) {
            IconButton(onClick = onToggleExpand, modifier = Modifier.align(Alignment.CenterEnd).padding(end = 8.dp)) {
                Icon(
                    if (expanded) Icons.Filled.FullscreenExit else Icons.Filled.Fullscreen,
                    if (expanded) "Shrink call" else "Expand call",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}

@Composable
private fun ControlButton(
    icon: ImageVector,
    contentDescription: String,
    active: Boolean = false,
    enabled: Boolean = true,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        enabled = enabled,
        shape = if (active) RoundedCornerShape(16.dp) else CircleShape,
        color = if (active) MaterialTheme.colorScheme.errorContainer else Color.Transparent,
        contentColor = if (active) MaterialTheme.colorScheme.onErrorContainer else MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.size(48.dp)
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(
                icon,
                contentDescription,
                modifier = Modifier.size(22.dp),
                tint = if (enabled) LocalContentColor.current else LocalContentColor.current.copy(alpha = 0.38f)
            )
        }
    }
}

@Composable
private fun VoiceEmptyState(channel: Channel, isJoined: Boolean) {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = Icons.Filled.Headphones,
            contentDescription = null,
            modifier = Modifier.size(64.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
        )
        Spacer(Modifier.height(16.dp))
        Text(
            text = if (isJoined) "Connected · Waiting for others" else "No one is here yet",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        if (channel.type == 13) {
            Text(
                text = "Stage channels do not support DAVE calls here.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = 8.dp)
            )
        }
    }
}

@Composable
private fun VoiceJoinBar(
    channel: Channel,
    voiceStore: VoiceStore,
    onCollapse: (() -> Unit)?
) {
    val join = rememberVoiceJoin(voiceStore)
    Surface(
        color = MaterialTheme.colorScheme.surfaceContainer,
        tonalElevation = 4.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            if (onCollapse != null) {
                IconButton(onClick = onCollapse) {
                    Icon(Icons.Filled.ExpandMore, "Collapse call", tint = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            Column(Modifier.weight(1f)) {
                Text(
                    text = "You are not in this call",
                    style = MaterialTheme.typography.bodyMedium
                )
                Text(
                    text = "Audio is sent only after DAVE encryption is ready.",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Button(
                onClick = { join(channel, false) },
                colors = ButtonDefaults.buttonColors(
                    containerColor = SpeakingColor,
                    contentColor = Color.White
                )
            ) {
                Icon(Icons.Filled.Call, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(Modifier.width(8.dp))
                Text(if (voiceStore.activeChannel == null) "Join Voice" else "Switch")
            }
        }
    }
}
