package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ApplicationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.delay
import org.koin.compose.koinInject
import kotlin.time.Duration.Companion.milliseconds

import me.lampu.lampcord.shared.model.customEmojiCdnUrl
import me.lampu.lampcord.shared.model.getDisplayUrl
import me.lampu.lampcord.shared.model.toTwemojiUrl
import me.lampu.lampcord.shared.ui.kit.clickableCursor

private fun getAssetUrl(applicationId: String?, assetId: String?): String? {
    if (assetId == null) return null
    if (assetId.startsWith("http")) return assetId
    
    if (assetId.contains(":")) {
        val parts = assetId.split(":", limit = 2)
        val platform = parts[0].lowercase()
        val id = parts[1]
        
        return when (platform) {
            "spotify" -> "https://i.scdn.co/image/$id"
            "mp" -> "https://media.discordapp.net/${if (id.startsWith("external/")) id else "external/$id"}"
            "youtube" -> "https://i.ytimg.com/vi/$id/hqdefault.jpg"
            else -> null
        }
    }
    
    if (assetId.startsWith("external/")) {
        return "https://media.discordapp.net/$assetId"
    }

    if (applicationId != null) {
        return "https://cdn.discordapp.com/app-assets/$applicationId/$assetId.jpg?size=512"
    }
    
    return null
}

@Composable
fun ElapsedTime(start: Long, modifier: Modifier = Modifier) {
    var elapsedText by remember(start) { mutableStateOf("") }

    val isSeconds = start < 10000000000L
    val startMs = if (isSeconds) start * 1000 else start

    LaunchedEffect(startMs) {
        while (true) {
            val now = getCurrentTimeMillis()
            val diff = (now - startMs).coerceAtLeast(0)
            val totalSeconds = diff / 1000
            val seconds = totalSeconds % 60
            val totalMinutes = totalSeconds / 60
            val minutes = totalMinutes % 60
            val hours = totalMinutes / 60
            
            elapsedText = if (hours > 0) {
                "${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}"
            } else {
                "${minutes}:${seconds.toString().padStart(2, '0')}"
            }
            delay(1000.milliseconds)
        }
    }

    if (elapsedText.isNotEmpty()) {
        Text(
            text = "for $elapsedText",
            style = MaterialTheme.typography.labelSmall,
            color = LocalContentColor.current.copy(alpha = 0.7f),
            modifier = modifier
        )
    }
}

@Composable
fun UserActivity(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    if (compact) {
        ActivityInnerContent(activity, modifier, compact)
    } else {
        Surface(
            modifier = modifier.fillMaxWidth(),
            shape = RoundedCornerShape(8.dp),
            color = MaterialTheme.colorScheme.surfaceContainerLow.copy(alpha = 0.4f),
        ) {
            ActivityInnerContent(activity, Modifier.padding(12.dp), compact)
        }
    }
}

@Composable
private fun ActivityInnerContent(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    when (activity.type) {
        4 -> CustomStatus(activity, modifier, compact)
        2 -> MusicActivity(activity, modifier, compact)
        else -> DefaultActivity(activity, modifier, compact)
    }
}

@Composable
fun CustomStatus(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    var isToggled by remember { mutableStateOf(false) }
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()

    Row(
        modifier = modifier
            .hoverable(interactionSource)
            .clickableCursor(interactionSource = interactionSource, indication = null) { isToggled = !isToggled },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        if (activity.emoji != null) {
            val isAnimated = activity.emoji.animated == true && !Settings.shared.reduceMotion
            val emojiUrl = activity.emoji.id?.let { customEmojiCdnUrl(it, isAnimated, size = 64) }
            
            if (emojiUrl != null) {
                AsyncImage(
                    model = emojiUrl,
                    contentDescription = activity.emoji.name,
                    allowAnimation = isAnimated,
                    modifier = Modifier.size(20.dp)
                )
            } else if (activity.emoji.name != null) {
                val displayUrl = activity.emoji.getDisplayUrl()
                if (displayUrl != null) {
                    AsyncImage(
                        model = displayUrl,
                        contentDescription = activity.emoji.name,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
        }

        if (!activity.state.isNullOrBlank()) {
            Text(
                text = activity.state,
                style = if (compact) MaterialTheme.typography.labelSmall else MaterialTheme.typography.bodyMedium,
                color = LocalContentColor.current.copy(alpha = 0.8f),
                maxLines = if (isHovered || isToggled) Int.MAX_VALUE else 2,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

@Composable
fun MusicActivity(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    val contentColor = LocalContentColor.current
    var isToggled by remember { mutableStateOf(false) }
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val isExpanded = isHovered || isToggled

    if (compact) {
        Row(
            modifier = modifier
                .hoverable(interactionSource)
                .clickableCursor(interactionSource = interactionSource, indication = null) { isToggled = !isToggled },
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Icon(
                imageVector = Icons.Filled.MusicNote2,
                contentDescription = null,
                modifier = Modifier.size(14.dp),
                tint = contentColor.copy(alpha = 0.7f)
            )
            Text(
                text = "Listening to ${activity.name}",
                style = MaterialTheme.typography.labelSmall,
                color = contentColor.copy(alpha = 0.7f),
                maxLines = if (isExpanded) Int.MAX_VALUE else 2,
                overflow = TextOverflow.Ellipsis
            )
        }
    } else {
        Column(modifier = modifier.fillMaxWidth()) {
            Text(
                text = "Listening to ${activity.name}",
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                color = contentColor.copy(alpha = 0.7f)
            )
            Spacer(Modifier.height(8.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                val imageUrl = getAssetUrl(activity.application_id, activity.assets?.large_image)
                val smallImageUrl = getAssetUrl(activity.application_id, activity.assets?.small_image)

                Box(modifier = Modifier.size(64.dp)) {
                    if (imageUrl != null) {
                        AsyncImage(
                            model = imageUrl,
                            contentDescription = activity.assets?.large_text,
                            modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(8.dp)),
                            contentScale = ContentScale.Crop
                        )
                    } else {
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(contentColor.copy(alpha = 0.1f), RoundedCornerShape(8.dp)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(Icons.Filled.Album, null, tint = contentColor.copy(alpha = 0.5f))
                        }
                    }

                    if (smallImageUrl != null) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .offset(x = 4.dp, y = 4.dp)
                                .size(24.dp)
                                .background(Color.Black, CircleShape)
                                .padding(2.dp)
                        ) {
                            AsyncImage(
                                model = smallImageUrl,
                                contentDescription = activity.assets?.small_text,
                                modifier = Modifier.fillMaxSize().clip(CircleShape),
                                contentScale = ContentScale.Crop
                            )
                        }
                    }
                }

                Spacer(Modifier.width(12.dp))

                Column(
                    modifier = Modifier
                        .hoverable(interactionSource)
                        .clickableCursor(interactionSource = interactionSource, indication = null) { isToggled = !isToggled }
                ) {
                    Text(
                        text = activity.details ?: "Unknown Track",
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.Bold,
                        color = contentColor,
                        maxLines = if (isExpanded) Int.MAX_VALUE else 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = "by ${activity.state ?: "Unknown Artist"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = contentColor.copy(alpha = 0.7f),
                        maxLines = if (isExpanded) Int.MAX_VALUE else 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (!activity.assets?.large_text.isNullOrBlank()) {
                        Text(
                            text = "on ${activity.assets.large_text}",
                            style = MaterialTheme.typography.bodySmall,
                            color = contentColor.copy(alpha = 0.5f),
                            maxLines = if (isExpanded) Int.MAX_VALUE else 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }

                    activity.timestamps?.let { timestamps ->
                        if (timestamps.start != null && timestamps.end == null) {
                            ElapsedTime(start = timestamps.start, modifier = Modifier.padding(top = 2.dp))
                        }
                    }
                }
            }

            activity.timestamps?.let { timestamps ->
                val start = timestamps.start
                val end = timestamps.end
                if (start != null && end != null) {
                    Spacer(Modifier.height(12.dp))
                    MusicProgressBar(
                        start = start,
                        end = end,
                        color = if (activity.name == "Spotify") Color(0xFF1DB954) else contentColor
                    )
                }
            }
        }
    }
}

@Composable
fun MusicProgressBar(start: Long, end: Long, color: Color = Color.White) {
    val total = (end - start).coerceAtLeast(1)
    
    val isSeconds = start < 10000000000L
    val startMs = if (isSeconds) start * 1000 else start
    val totalMs = if (isSeconds) total * 1000 else total

    var currentMillis by remember(start, end) { 
        val now = getCurrentTimeMillis()
        mutableLongStateOf((now - startMs).coerceIn(0, totalMs)) 
    }
    
    LaunchedEffect(start, end) {
        while (currentMillis < totalMs) {
            val now = getCurrentTimeMillis()
            currentMillis = (now - startMs).coerceIn(0, totalMs)
            if (currentMillis < totalMs) {
                delay(1000.milliseconds)
            }
        }
    }

    val progress = currentMillis.toFloat() / totalMs.toFloat()

    fun formatTime(ms: Long): String {
        val totalSeconds = ms / 1000
        val minutes = totalSeconds / 60
        val seconds = totalSeconds % 60
        return "$minutes:${seconds.toString().padStart(2, '0')}"
    }

    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        LinearProgressIndicator(
            progress = { progress },
            modifier = Modifier.fillMaxWidth().height(4.dp).clip(CircleShape),
            color = color,
            trackColor = color.copy(alpha = 0.2f),
        )
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(formatTime(currentMillis), style = MaterialTheme.typography.labelSmall, color = LocalContentColor.current.copy(alpha = 0.7f))
            Text(formatTime(totalMs), style = MaterialTheme.typography.labelSmall, color = LocalContentColor.current.copy(alpha = 0.7f))
        }
    }
}

@Composable
fun DefaultActivity(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    val contentColor = LocalContentColor.current
    val applicationStore: ApplicationStore = koinInject()
    var isToggled by remember { mutableStateOf(false) }
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val isExpanded = isHovered || isToggled
    
    val verb = when (activity.type) {
        0 -> "Playing"
        1 -> "Streaming"
        2 -> "Listening to"
        3 -> "Watching"
        5 -> "Competing in"
        else -> "Playing"
    }

    if (compact) {
        Row(
            modifier = modifier
                .hoverable(interactionSource)
                .clickableCursor(interactionSource = interactionSource, indication = null) { isToggled = !isToggled },
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Icon(
                imageVector = when (activity.type) {
                    1 -> Icons.Filled.Videocam
                    3 -> Icons.Filled.PlayArrow
                    5 -> Icons.Filled.Star
                    else -> Icons.Filled.SportsEsports
                },
                contentDescription = null,
                modifier = Modifier.size(14.dp),
                tint = contentColor.copy(alpha = 0.7f)
            )
            Text(
                text = "$verb ${activity.name}",
                style = MaterialTheme.typography.labelSmall,
                color = contentColor.copy(alpha = 0.7f),
                maxLines = if (isExpanded) Int.MAX_VALUE else 2,
                overflow = TextOverflow.Ellipsis
            )
        }
    } else {
        Column(modifier = modifier.fillMaxWidth()) {
            Text(
                text = verb,
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                color = contentColor.copy(alpha = 0.7f)
            )
            Spacer(Modifier.height(8.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                val imageUrl = remember(activity.application_id, activity.assets?.large_image) {
                    getAssetUrl(activity.application_id, activity.assets?.large_image)
                }
                
                val appIconUrl = if (imageUrl == null && activity.application_id != null) {
                    val app = applicationStore.getApplication(activity.application_id)
                    if (app?.icon != null) {
                        "https://cdn.discordapp.com/app-icons/${activity.application_id}/${app.icon}.webp?size=512"
                    } else null
                } else null

                val finalImageUrl = imageUrl ?: appIconUrl

                val smallImageUrl = getAssetUrl(activity.application_id, activity.assets?.small_image)

                Box(modifier = Modifier.size(64.dp)) {
                    if (finalImageUrl != null) {
                        AsyncImage(
                            model = finalImageUrl,
                            contentDescription = activity.assets?.large_text,
                            modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(8.dp)),
                            contentScale = ContentScale.Crop
                        )
                    } else {
                        Box(
                            modifier = Modifier
                                .fillMaxSize()
                                .background(contentColor.copy(alpha = 0.1f), RoundedCornerShape(8.dp)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                imageVector = when (activity.type) {
                                    1 -> Icons.Filled.Radio
                                    3 -> Icons.Filled.PlayArrow
                                    5 -> Icons.Filled.Star
                                    else -> Icons.Filled.SportsEsports
                                },
                                contentDescription = null,
                                tint = contentColor.copy(alpha = 0.5f)
                            )
                        }
                    }

                    if (smallImageUrl != null) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.BottomEnd)
                                .offset(x = 4.dp, y = 4.dp)
                                .size(24.dp)
                                .background(Color.Black, CircleShape)
                                .padding(2.dp)
                        ) {
                            AsyncImage(
                                model = smallImageUrl,
                                contentDescription = activity.assets?.small_text,
                                modifier = Modifier.fillMaxSize().clip(CircleShape),
                                contentScale = ContentScale.Crop
                            )
                        }
                    }
                }
                
                Spacer(Modifier.width(12.dp))

                Column(
                    modifier = Modifier
                        .hoverable(interactionSource)
                        .clickableCursor(interactionSource = interactionSource, indication = null) { isToggled = !isToggled }
                ) {
                    Text(
                        text = activity.name,
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.Bold,
                        color = contentColor,
                        maxLines = if (isExpanded) Int.MAX_VALUE else 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (!activity.details.isNullOrBlank()) {
                        Text(
                            text = activity.details,
                            style = MaterialTheme.typography.bodyMedium,
                            color = contentColor.copy(alpha = 0.7f),
                            maxLines = if (isExpanded) Int.MAX_VALUE else 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    if (!activity.state.isNullOrBlank()) {
                        Text(
                            text = activity.state,
                            style = MaterialTheme.typography.bodySmall,
                            color = contentColor.copy(alpha = 0.5f),
                            maxLines = if (isExpanded) Int.MAX_VALUE else 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    
                    activity.timestamps?.let { timestamps ->
                        if (timestamps.start != null && timestamps.end == null) {
                            ElapsedTime(start = timestamps.start, modifier = Modifier.padding(top = 2.dp))
                        }
                    }
                }
            }

            activity.timestamps?.let { timestamps ->
                val start = timestamps.start
                val end = timestamps.end
                if (start != null && end != null) {
                    Spacer(Modifier.height(12.dp))
                    MusicProgressBar(
                        start = start,
                        end = end,
                        color = contentColor
                    )
                }
            }
        }
    }
}
