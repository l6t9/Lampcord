package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.BoardApplication
import me.lampu.lampcord.shared.api.BoardGameEntry
import me.lampu.lampcord.shared.api.BoardGameInfo
import me.lampu.lampcord.shared.api.BoardWidget
import me.lampu.lampcord.shared.api.ProfileBoard
import me.lampu.lampcord.shared.api.ProfileBoardApi
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private val WIDGET_TITLES = mapOf(
    "current_games" to "Playing",
    "played_games" to "Recently Played",
    "favorite_games" to "Favorites",
    "want_to_play_games" to "Want to Play",
    "application" to "Application"
)

private val GAME_TAG_INFO: Map<String, Pair<String, ImageVector>> = mapOf(
    "noob" to ("Noob" to Icons.Rounded.ChildCare),
    "learning_the_ropes" to ("Learning the ropes" to Icons.Rounded.MenuBook),
    "casual" to ("Casual" to Icons.Rounded.Weekend),
    "getting_good" to ("Getting good" to Icons.Rounded.TrendingUp),
    "intermediate" to ("Intermediate" to Icons.Rounded.SignalCellularAlt),
    "expert" to ("Expert" to Icons.Rounded.WorkspacePremium),
    "better_than_you" to ("Better than you" to Icons.Rounded.WorkspacePremium),
    "obsessed" to ("Obsessed" to Icons.Rounded.Favorite),
    "love_it" to ("Love it" to Icons.Rounded.Favorite),
    "kind_of_love_it" to ("Kind of love it" to Icons.Rounded.ThumbUp),
    "kind_of_hate_it" to ("Kind of hate it" to Icons.Rounded.ThumbDown),
    "rage_quitting" to ("Rage quitting" to Icons.Rounded.SentimentVeryDissatisfied),
    "like_it" to ("Like it" to Icons.Rounded.ThumbUp),
    "frustrated" to ("Frustrated" to Icons.Rounded.SentimentDissatisfied),
    "too_easy" to ("Too easy" to Icons.Rounded.SentimentVerySatisfied),
    "looking_for_group" to ("Looking for group" to Icons.Rounded.GroupAdd),
    "open_to_play" to ("Open to play" to Icons.Rounded.SportsEsports),
    "looking_for_tips" to ("Looking for tips" to Icons.Rounded.Lightbulb),
    "open_to_teach" to ("Open to teach" to Icons.Rounded.School),
    "looking_to_discuss" to ("Looking to discuss" to Icons.Rounded.Forum)
)

@Composable
fun ProfileBoardContent(
    profile: UserProfile,
    modifier: Modifier = Modifier,
    boardApi: ProfileBoardApi = koinInject(),
    loadedBoard: ProfileBoard? = null,
    onBoardLoaded: (ProfileBoard?) -> Unit = {}
) {
    val userId = profile.user.id
    var board by remember(userId) { mutableStateOf<ProfileBoard?>(null) }
    var failed by remember(userId) { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    val effectiveBoard = loadedBoard ?: board

    LaunchedEffect(userId) {
        if (loadedBoard == null) {
            val loaded = boardApi.load(userId)
            if (loaded == null) failed = true else board = loaded
        }
    }

    LaunchedEffect(effectiveBoard) {
        onBoardLoaded(effectiveBoard)
    }

    when (val loaded = effectiveBoard) {
        null -> if (failed) {
            BoardMessage("Could not load this profile's board.")
        } else {
            Box(modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        }

        else -> if (loaded.isEmpty) {
            BoardMessage("This profile has not set up a board.")
        } else {
            Column(
                modifier = modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(24.dp)
            ) {
                loaded.widgets.forEach { widget ->
                    key(widget.type + (widget.applicationId ?: "")) {
                        BoardWidgetSection(
                            title = WIDGET_TITLES[widget.type] ?: "Board",
                            widget = widget,
                            board = loaded,
                            onRetry = { scope.launch { board = boardApi.load(userId) } }
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun BoardMessage(text: String) {
    Box(modifier = Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
        Text(text, color = LocalContentColor.current.copy(alpha = 0.7f))
    }
}

@Composable
private fun BoardWidgetSection(
    title: String,
    widget: BoardWidget,
    board: ProfileBoard,
    onRetry: () -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.SemiBold,
            color = LocalContentColor.current
        )
        widget.applicationId?.let { id ->
            board.applications[id]?.let { ApplicationBoard(it) }
                ?: ApplicationBoardMissing(onRetry)
        }
        if (widget.games.isNotEmpty()) {
            LazyRow(
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                contentPadding = PaddingValues(end = 4.dp)
            ) {
                items(widget.games, key = { it.gameId }) { entry ->
                    GameWidgetCard(entry, board.games[entry.gameId])
                }
            }
        }
    }
}

@Composable
private fun GameWidgetCard(entry: BoardGameEntry, info: BoardGameInfo?) {
    val name = info?.name ?: entry.gameId
    val comment = entry.comment?.takeIf { it.isNotBlank() }
    val tags = entry.tags
    val cardShape = RoundedCornerShape(20.dp)
    val hasImage = info?.image != null

    Surface(
        modifier = Modifier.width(320.dp),
        shape = cardShape,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        tonalElevation = 2.dp
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            horizontalArrangement = Arrangement.spacedBy(14.dp),
            verticalAlignment = Alignment.Top
        ) {
            Surface(
                modifier = Modifier
                    .width(92.dp)
                    .height(138.dp),
                shape = RoundedCornerShape(10.dp),
                color = MaterialTheme.colorScheme.surfaceContainerHighest
            ) {
                if (hasImage) {
                    AsyncImage(
                        model = info?.image,
                        contentDescription = name,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        showPlaceholder = true
                    )
                } else {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(
                                Brush.linearGradient(
                                    listOf(
                                        MaterialTheme.colorScheme.primaryContainer,
                                        MaterialTheme.colorScheme.secondaryContainer
                                    )
                                )
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = name.take(1).uppercase(),
                            style = MaterialTheme.typography.displaySmall,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onPrimaryContainer
                        )
                    }
                }
            }

            Column(
                modifier = Modifier
                    .weight(1f)
                    .heightIn(min = 138.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text(
                    text = name,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
                if (tags.isNotEmpty()) {
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        tags.forEach { tag -> GameTag(tag) }
                    }
                }
                if (comment != null) {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalAlignment = Alignment.Top
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.FormatQuote,
                            contentDescription = null,
                            modifier = Modifier.size(14.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text(
                            text = comment,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun GameTag(tag: String) {
    val info = GAME_TAG_INFO[tag]
    val label = info?.first ?: tag.replace('_', ' ').replaceFirstChar { it.uppercase() }
    val icon = info?.second

    Surface(
        shape = CircleShape,
        color = MaterialTheme.colorScheme.surfaceContainerHighest,
        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.widthIn(max = 150.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            if (icon != null) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(12.dp)
                )
            }
            Text(
                text = label,
                style = MaterialTheme.typography.labelSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

@Composable
private fun ApplicationBoard(application: BoardApplication) {
    val shape = RoundedCornerShape(28.dp)
    Surface(
        shape = shape,
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Surface(
                    shape = RoundedCornerShape(20.dp),
                    color = MaterialTheme.colorScheme.surfaceContainerHighest,
                    modifier = Modifier.size(72.dp)
                ) {
                    AsyncImage(
                        model = application.icon,
                        contentDescription = application.name,
                        modifier = Modifier.fillMaxSize(),
                        contentScale = ContentScale.Crop,
                        showPlaceholder = application.icon != null
                    )
                }
                Spacer(Modifier.width(16.dp))
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(3.dp)
                ) {
                    Text(
                        application.name,
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                    application.title?.let {
                        Text(
                            it,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    application.subtitles.filter { it.isNotBlank() }.forEach {
                        Text(
                            it,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            if (application.stats.isNotEmpty()) {
                LazyRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    items(application.stats, key = { it.label }) { stat ->
                        Surface(
                            shape = RoundedCornerShape(18.dp),
                            color = MaterialTheme.colorScheme.secondaryContainer,
                            contentColor = MaterialTheme.colorScheme.onSecondaryContainer
                        ) {
                            Column(Modifier.padding(horizontal = 14.dp, vertical = 10.dp)) {
                                Text(
                                    stat.value,
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold
                                )
                                Text(
                                    stat.label,
                                    style = MaterialTheme.typography.labelSmall
                                )
                            }
                        }
                    }
                }
            }
            application.image?.let { image ->
                AsyncImage(
                    model = image,
                    contentDescription = null,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(150.dp)
                        .clip(RoundedCornerShape(20.dp)),
                    contentScale = ContentScale.Crop
                )
            }
        }
    }
}

@Composable
private fun ApplicationBoardMissing(onRetry: () -> Unit) {
    Surface(
        onClick = onRetry,
        shape = RoundedCornerShape(28.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Box(Modifier.fillMaxWidth().padding(20.dp), contentAlignment = Alignment.Center) {
            Text(
                "Application details unavailable. Tap to retry.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}
