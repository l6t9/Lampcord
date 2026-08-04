package com.example.lampcord.shared.ui.components

import androidx.compose.foundation.background
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
import androidx.compose.ui.unit.sp
import com.example.lampcord.shared.model.Activity
import com.example.lampcord.shared.ui.icons.Icons
import com.example.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.delay

private fun getAssetUrl(applicationId: String?, assetId: String?): String? {
    if (assetId == null) return null
    if (assetId.startsWith("spotify:")) {
        return "https://i.scdn.co/image/${assetId.removePrefix("spotify:")}"
    }
    if (assetId.startsWith("mp:external/")) {
        return "https://media.discordapp.net/external/${assetId.removePrefix("mp:external/")}"
    }
    if (applicationId != null) {
        return "https://cdn.discordapp.com/app-assets/$applicationId/$assetId.png"
    }
    return null
}

@Composable
fun UserActivity(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    when (activity.type) {
        4 -> CustomStatus(activity, modifier, compact)
        2 -> MusicActivity(activity, modifier, compact)
        else -> DefaultActivity(activity, modifier, compact)
    }
}

@Composable
fun CustomStatus(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        if (activity.emoji != null) {
            val emojiUrl = if (activity.emoji.id != null) {
                "https://cdn.discordapp.com/emojis/${activity.emoji.id}.${if (activity.emoji.animated == true) "gif" else "png"}?size=32"
            } else null
            
            if (emojiUrl != null) {
                AsyncImage(
                    model = emojiUrl,
                    contentDescription = activity.emoji.name,
                    modifier = Modifier.size(20.dp)
                )
            } else if (activity.emoji.name != null) {
                Text(activity.emoji.name, fontSize = 16.sp)
            }
        }
        
        if (!activity.state.isNullOrBlank()) {
            Text(
                text = activity.state,
                style = if (compact) MaterialTheme.typography.labelSmall else MaterialTheme.typography.bodyMedium,
                color = LocalContentColor.current.copy(alpha = 0.8f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}

@Composable
fun MusicActivity(activity: Activity, modifier: Modifier = Modifier, compact: Boolean = false) {
    val contentColor = LocalContentColor.current
    if (compact) {
        Row(
            modifier = modifier,
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
                maxLines = 1,
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

                Column {
                    Text(
                        text = activity.details ?: "Unknown Track",
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.Bold,
                        color = contentColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = "by ${activity.state ?: "Unknown Artist"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = contentColor.copy(alpha = 0.7f),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (!activity.assets?.large_text.isNullOrBlank()) {
                        Text(
                            text = "on ${activity.assets.large_text}",
                            style = MaterialTheme.typography.bodySmall,
                            color = contentColor.copy(alpha = 0.5f),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
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
        while (true) {
            val now = getCurrentTimeMillis()
            currentMillis = (now - startMs).coerceIn(0, totalMs)
            delay(1000)
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
    val verb = when (activity.type) {
        0 -> "Playing"
        1 -> "Streaming"
        3 -> "Watching"
        5 -> "Competing in"
        else -> "Playing"
    }

    if (compact) {
        Text(
            text = "$verb ${activity.name}",
            style = MaterialTheme.typography.labelSmall,
            color = contentColor.copy(alpha = 0.7f),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = modifier
        )
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
                val imageUrl = getAssetUrl(activity.application_id, activity.assets?.large_image)
                val smallImageUrl = getAssetUrl(activity.application_id, activity.assets?.small_image)

                if (imageUrl != null) {
                    Box(modifier = Modifier.size(64.dp)) {
                        AsyncImage(
                            model = imageUrl,
                            contentDescription = activity.assets?.large_text,
                            modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(8.dp)),
                            contentScale = ContentScale.Crop
                        )
                        
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
                }

                Column {
                    Text(
                        text = activity.name,
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.Bold,
                        color = contentColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (!activity.details.isNullOrBlank()) {
                        Text(
                            text = activity.details,
                            style = MaterialTheme.typography.bodyMedium,
                            color = contentColor.copy(alpha = 0.7f),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    if (!activity.state.isNullOrBlank()) {
                        Text(
                            text = activity.state,
                            style = MaterialTheme.typography.bodySmall,
                            color = contentColor.copy(alpha = 0.5f),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }
        }
    }
}
