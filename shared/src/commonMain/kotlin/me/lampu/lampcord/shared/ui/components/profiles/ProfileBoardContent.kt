package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
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
import org.koin.compose.koinInject

private val WIDGET_TITLES = mapOf(
    "current_games" to "Playing",
    "played_games" to "Recently Played",
    "favorite_games" to "Favorites",
    "want_to_play_games" to "Want to Play",
    "application" to "Application"
)

@Composable
fun ProfileBoardContent(
    profile: UserProfile,
    modifier: Modifier = Modifier,
    boardApi: ProfileBoardApi = koinInject()
) {
    val userId = profile.user.id
    var board by remember(userId) { mutableStateOf<ProfileBoard?>(null) }
    var failed by remember(userId) { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(userId) {
        val loaded = boardApi.load(userId)
        if (loaded == null) failed = true else board = loaded
    }

    when (val loaded = board) {
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
            LazyColumn(
                modifier = modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(20.dp)
            ) {
                loaded.widgets.forEach { widget ->
                    item(key = widget.type + (widget.applicationId ?: "")) {
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
        Text(text, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun BoardWidgetSection(
    title: String,
    widget: BoardWidget,
    board: ProfileBoard,
    onRetry: () -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(
            text = title.uppercase(),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        widget.applicationId?.let { id ->
            board.applications[id]?.let { ApplicationBoard(it) }
                ?: ApplicationBoardMissing(onRetry)
        }
        if (widget.games.isNotEmpty()) {
            LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                items(widget.games, key = { it.gameId }) { entry ->
                    GameTile(entry, board.games[entry.gameId])
                }
            }
        }
    }
}

@Composable
private fun GameTile(entry: BoardGameEntry, info: BoardGameInfo?) {
    val name = info?.name ?: entry.gameId
    Column(
        modifier = Modifier.width(132.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        AsyncImage(
            model = info?.image,
            contentDescription = name,
            modifier = Modifier.size(96.dp).clip(RoundedCornerShape(16.dp)),
            contentScale = ContentScale.Crop,
            showPlaceholder = info?.image != null
        )
        Spacer(Modifier.height(6.dp))
        Text(
            text = name,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.SemiBold,
            maxLines = 2
        )
        if (!entry.comment.isNullOrBlank()) {
            Text(
                text = entry.comment,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2
            )
        }
    }
}

@Composable
private fun ApplicationBoard(application: BoardApplication) {
    Surface(
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            AsyncImage(
                model = application.icon,
                contentDescription = application.name,
                modifier = Modifier.size(64.dp).clip(RoundedCornerShape(12.dp)),
                contentScale = ContentScale.Crop,
                showPlaceholder = application.icon != null
            )
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(application.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                application.title?.let {
                    Text(
                        it,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                application.subtitles.filter { it.isNotBlank() }.forEach {
                    Text(
                        it,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
    if (application.stats.isNotEmpty()) {
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(application.stats, key = { it.label }) { stat ->
                Surface(
                    shape = RoundedCornerShape(8.dp),
                    color = MaterialTheme.colorScheme.surfaceContainerHighest
                ) {
                    Column(Modifier.padding(horizontal = 12.dp, vertical = 8.dp)) {
                        Text(stat.value, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        Text(
                            stat.label,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
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
            modifier = Modifier.fillMaxWidth().height(120.dp).clip(RoundedCornerShape(12.dp)),
            contentScale = ContentScale.Crop
        )
    }
}

@Composable
private fun ApplicationBoardMissing(onRetry: () -> Unit) {
    Surface(
        onClick = onRetry,
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Box(Modifier.fillMaxWidth().padding(16.dp), contentAlignment = Alignment.Center) {
            Text(
                "Application details unavailable. Tap to retry.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}