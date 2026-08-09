package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.LocalMedia
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getLocalMedia
import me.lampu.lampcord.shared.utils.getLocalMediaBytes
import me.lampu.lampcord.shared.utils.getPlatformName
import kotlinx.coroutines.launch

@Composable
fun MediaPicker(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    var mediaList by remember { mutableStateOf<List<LocalMedia>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var hasPermission by remember { mutableStateOf(true) }
    var permissionRequested by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    if (!permissionRequested) {
        me.lampu.lampcord.shared.utils.RequestMediaPermissions { granted ->
            hasPermission = granted
            permissionRequested = true
            if (granted) {
                scope.launch {
                    try {
                        mediaList = getLocalMedia()
                    } catch (e: Exception) {
                        hasPermission = false
                    }
                    isLoading = false
                }
            } else {
                isLoading = false
            }
        }
    }

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .height(400.dp),
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Column {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp)
            ) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopCenter)
                        .padding(top = 4.dp)
                        .size(width = 32.dp, height = 4.dp)
                        .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f), CircleShape)
                )

                Text(
                    "Select Media",
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.align(Alignment.Center).padding(top = 12.dp)
                )
                TextButton(
                    onClick = onDismiss,
                    modifier = Modifier.align(Alignment.CenterEnd).padding(top = 12.dp)
                ) {
                    Text("Done")
                }
            }

            if (!hasPermission) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Permission denied", color = MaterialTheme.colorScheme.error)
                        Button(onClick = { /* Could trigger permission request again */ }) {
                            Text("Retry")
                        }
                    }
                }
            } else if (isLoading) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            } else if (mediaList.isEmpty()) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text("No media found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            } else {
                LazyVerticalGrid(
                    columns = GridCells.Fixed(3),
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(1.dp),
                    horizontalArrangement = Arrangement.spacedBy(1.dp),
                    verticalArrangement = Arrangement.spacedBy(1.dp)
                ) {
                    items(mediaList, key = { it.id }) { media ->
                        val isSelected = chatState.pendingFiles.any { it.first == media.name }
                        
                        Box(
                            modifier = Modifier
                                .aspectRatio(1f)
                                .clickable {
                                    scope.launch {
                                        if (isSelected) {
                                            chatState.pendingFiles.removeAll { it.first == media.name }
                                        } else {
                                            val bytes = getLocalMediaBytes(media.uri)
                                            if (bytes != null) {
                                                chatState.pendingFiles.add(media.name to bytes)
                                            }
                                        }
                                    }
                                }
                        ) {
                            AsyncImage(
                                model = media.uri,
                                contentDescription = media.name,
                                contentScale = ContentScale.Crop,
                                modifier = Modifier.fillMaxSize()
                            )

                            if (media.isVideo) {
                                Icon(
                                    imageVector = Icons.Filled.PlayArrow,
                                    contentDescription = "Video",
                                    tint = Color.White,
                                    modifier = Modifier
                                        .align(Alignment.BottomStart)
                                        .padding(4.dp)
                                        .size(16.dp)
                                        .background(Color.Black.copy(alpha = 0.4f), CircleShape)
                                )
                            }

                            if (isSelected) {
                                Box(
                                    modifier = Modifier
                                        .fillMaxSize()
                                        .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.3f))
                                ) {
                                    Icon(
                                        imageVector = Icons.Filled.Check,
                                        contentDescription = "Selected",
                                        tint = Color.White,
                                        modifier = Modifier
                                            .align(Alignment.TopEnd)
                                            .padding(4.dp)
                                            .size(20.dp)
                                            .background(MaterialTheme.colorScheme.primary, CircleShape)
                                            .padding(2.dp)
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
