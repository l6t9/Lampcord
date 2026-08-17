package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.animation.core.animateDp
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FloatingToolbarDefaults
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupProperties
import androidx.compose.ui.zIndex
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.model.LocalMedia
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.model.PollAnswer
import me.lampu.lampcord.shared.model.PollMedia
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.AdaptiveModalBottomSheet
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.VideoThumbnail
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.RequestMediaPermissions
import me.lampu.lampcord.shared.utils.getLocalFiles
import me.lampu.lampcord.shared.utils.getLocalMedia
import me.lampu.lampcord.shared.utils.getLocalMediaBytes
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlin.time.Duration.Companion.milliseconds

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun MediaPicker(
    onDismiss: () -> Unit,
    messageStore: MessageStore = koinInject()
) {
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"
    var selectedTab by remember { mutableStateOf(0) }
    var showSystemFilePicker by remember { mutableStateOf(false) }
    
    val scope = rememberCoroutineScope()
    var isToolbarVisible by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) {
        isToolbarVisible = true
    }

    val animatedDismiss = {
        scope.launch {
            isToolbarVisible = false
            delay(100.milliseconds)
            onDismiss()
        }
    }

    FilePicker(
        show = showSystemFilePicker,
        onFileSelected = { files ->
            messageStore.pendingFiles.addAll(files.map { PendingFile(it.first, it.second) })
        },
        onDismiss = { showSystemFilePicker = false }
    )

    if (isMobile) {
        AdaptiveModalBottomSheet(
            onDismissRequest = { animatedDismiss() },
            sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = false)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                MediaPickerContent(
                    onDismiss = onDismiss,
                    isMobile = true,
                    selectedTab = selectedTab,
                    onTabSelected = { selectedTab = it },
                    messageStore = messageStore
                )

                val transition = updateTransition(targetState = isToolbarVisible, label = "ToolbarTransition")
                val toolbarAlpha by transition.animateFloat(
                    transitionSpec = { if (targetState) tween(400) else tween(100) },
                    label = "alpha"
                ) { state -> if (state) 1f else 0f }
                val toolbarSlideOffset by transition.animateDp(
                    transitionSpec = { if (targetState) tween(400) else tween(100) },
                    label = "slide"
                ) { state -> if (state) 0.dp else 40.dp }

                Popup(
                    alignment = Alignment.BottomCenter,
                    properties = PopupProperties(
                        focusable = false,
                        dismissOnBackPress = false,
                        dismissOnClickOutside = false
                    )
                ) {
                    MediaPickerTabs(
                        selectedTab = selectedTab,
                        onTabSelected = { selectedTab = it },
                        onSystemPickerClick = { showSystemFilePicker = true },
                        modifier = Modifier
                            .graphicsLayer {
                                alpha = toolbarAlpha
                                translationY = toolbarSlideOffset.toPx()
                            }
                            .navigationBarsPadding()
                            .zIndex(1f)
                    )
                }
            }
        }
    } else {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .height(450.dp),
            shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
            color = MaterialTheme.colorScheme.surfaceContainerLow
        ) {
            MediaPickerContent(
                onDismiss = onDismiss,
                isMobile = false,
                selectedTab = selectedTab,
                onTabSelected = { selectedTab = it },
                onSystemPickerClick = { showSystemFilePicker = true }
            )
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun MediaPickerContent(
    onDismiss: () -> Unit,
    isMobile: Boolean,
    selectedTab: Int,
    onTabSelected: (Int) -> Unit,
    onSystemPickerClick: (() -> Unit)? = null,
    messageStore: MessageStore = koinInject()
) {
    val tabs = listOf("Images", "Files", "Camera", "Thread", "Poll")
    
    var mediaList by remember { mutableStateOf<List<LocalMedia>>(emptyList()) }
    var fileList by remember { mutableStateOf<List<LocalMedia>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var hasPermission by remember { mutableStateOf(true) }
    var permissionRequested by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    // Poll State
    var pollQuestion by remember { mutableStateOf("") }
    val pollAnswers = remember { mutableStateListOf("", "") }
    var pollAllowMultiselect by remember { mutableStateOf(false) }

    // Thread State
    var threadName by remember { mutableStateOf("") }

    LaunchedEffect(selectedTab, permissionRequested) {
        if (!permissionRequested) return@LaunchedEffect
        
        isLoading = true
        try {
            if (selectedTab == 0 && mediaList.isEmpty()) {
                mediaList = getLocalMedia()
            } else if (selectedTab == 1 && fileList.isEmpty()) {
                fileList = getLocalFiles()
            }
        } catch (e: Exception) {
            // Log or handle error
        } finally {
            isLoading = false
        }
    }

    if (!permissionRequested) {
        RequestMediaPermissions { granted ->
            hasPermission = granted
            permissionRequested = true
        }
    }

    Column(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 16.dp, end = 16.dp, top = if (isMobile) 0.dp else 8.dp, bottom = 0.dp)
        ) {
            if (!isMobile) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopCenter)
                        .padding(top = 4.dp)
                        .size(width = 32.dp, height = 4.dp)
                        .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f), CircleShape)
                )
            }

            Text(
                text = tabs[selectedTab],
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.align(Alignment.Center).padding(top = if (isMobile) 0.dp else 8.dp)
            )
            TextButton(
                onClick = onDismiss,
                modifier = Modifier.align(Alignment.CenterEnd).padding(top = if (isMobile) 0.dp else 8.dp)
            ) {
                Text("Done")
            }
        }

        Box(modifier = Modifier.weight(1f)) {
            when (selectedTab) {
                0 -> {
                    if (!hasPermission) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Text("Permission denied", color = MaterialTheme.colorScheme.error)
                                Button(onClick = { permissionRequested = false }) {
                                    Text("Retry")
                                }
                            }
                        }
                    } else if (isLoading) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            ContainedLoadingIndicator()
                        }
                    } else if (mediaList.isEmpty()) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Text("No media found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    } else {
                        LazyVerticalGrid(
                            columns = GridCells.Fixed(3),
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(bottom = 80.dp),
                            horizontalArrangement = Arrangement.spacedBy(1.dp),
                            verticalArrangement = Arrangement.spacedBy(1.dp)
                        ) {
                            items(mediaList, key = { it.id }) { media ->
                                val isSelected = messageStore.pendingFiles.any { it.name == media.name }
                                
                                Box(
                                    modifier = Modifier
                                        .aspectRatio(1f)
                                        .clickable {
                                            scope.launch {
                                                if (isSelected) {
                                                    messageStore.pendingFiles.removeAll { it.name == media.name }
                                                } else {
                                                    val bytes = getLocalMediaBytes(media.uri)
                                                    if (bytes != null) {
                                                        messageStore.pendingFiles.add(PendingFile(media.name, bytes, media.uri))
                                                    }
                                                }
                                            }
                                        }
                                ) {
                                    if (media.isVideo) {
                                        VideoThumbnail(
                                            uri = media.uri,
                                            contentDescription = media.name,
                                            modifier = Modifier.fillMaxSize()
                                        )
                                    } else {
                                        AsyncImage(
                                            model = media.uri,
                                            contentDescription = media.name,
                                            contentScale = ContentScale.Crop,
                                            modifier = Modifier.fillMaxSize()
                                        )
                                    }

                                    if (media.isVideo) {
                                        Box(
                                            modifier = Modifier
                                                .align(Alignment.BottomEnd)
                                                .padding(4.dp)
                                                .background(Color.Black.copy(alpha = 0.6f), RoundedCornerShape(4.dp))
                                                .padding(horizontal = 4.dp, vertical = 2.dp)
                                        ) {
                                            Text(
                                                text = formatDuration(media.duration ?: 0),
                                                color = Color.White,
                                                style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                                                fontWeight = FontWeight.Bold
                                            )
                                        }
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
                1 -> {
                    if (!hasPermission) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Text("Permission denied", color = MaterialTheme.colorScheme.error)
                                Button(onClick = { permissionRequested = false }) {
                                    Text("Retry")
                                }
                            }
                        }
                    } else if (isLoading) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            ContainedLoadingIndicator()
                        }
                    } else if (fileList.isEmpty()) {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Text("No files found", color = MaterialTheme.colorScheme.onSurfaceVariant)
                                Spacer(Modifier.height(8.dp))
                                Text(
                                    "Use the button below to browse all files",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                )
                            }
                        }
                    } else {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(bottom = 80.dp)
                        ) {
                            items(fileList.size, key = { fileList[it].id }) { index ->
                                val file = fileList[index]
                                val isSelected = messageStore.pendingFiles.any { it.name == file.name }
                                
                                FileItem(
                                    file = file,
                                    isSelected = isSelected,
                                    onClick = {
                                        scope.launch {
                                            if (isSelected) {
                                                messageStore.pendingFiles.removeAll { it.name == file.name }
                                            } else {
                                                val bytes = getLocalMediaBytes(file.uri)
                                                if (bytes != null) {
                                                    messageStore.pendingFiles.add(PendingFile(file.name, bytes, file.uri))
                                                }
                                            }
                                        }
                                    }
                                )
                            }
                        }
                    }
                }
                2 -> {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Icon(Icons.Rounded.Devices, null, modifier = Modifier.size(64.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f))
                            Spacer(Modifier.height(24.dp))
                            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                                Button(onClick = { /* Launch Camera Photo */ }) {
                                    Icon(Icons.Filled.PhotoCamera, null, modifier = Modifier.size(18.dp))
                                    Spacer(Modifier.width(8.dp))
                                    Text("Take Photo")
                                }
                                Button(onClick = { /* Launch Camera Video */ }) {
                                    Icon(Icons.Filled.VideoCall, null, modifier = Modifier.size(18.dp))
                                    Spacer(Modifier.width(8.dp))
                                    Text("Record Video")
                                }
                            }
                        }
                    }
                }
                3 -> {
                    Column(
                        modifier = Modifier.fillMaxSize().padding(16.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text("Create a new thread", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(16.dp))
                        OutlinedTextField(
                            value = threadName,
                            onValueChange = { threadName = it },
                            label = { Text("Thread Name") },
                            modifier = Modifier.fillMaxWidth(),
                            singleLine = true
                        )
                        Spacer(Modifier.height(16.dp))
                        Button(
                            onClick = { 
                                // Handle thread creation
                                // For now just placeholders as requested
                                onDismiss()
                            },
                            enabled = threadName.isNotBlank()
                        ) {
                            Text("Create Thread")
                        }
                    }
                }
                4 -> {
                    LazyColumn(
                        modifier = Modifier.fillMaxSize().padding(16.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        item {
                            Text("Create Poll", style = MaterialTheme.typography.titleMedium)
                        }
                        item {
                            OutlinedTextField(
                                value = pollQuestion,
                                onValueChange = { pollQuestion = it },
                                label = { Text("Question") },
                                modifier = Modifier.fillMaxWidth()
                            )
                        }
                        items(pollAnswers.size) { index ->
                            val answer = pollAnswers[index]
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                OutlinedTextField(
                                    value = answer,
                                    onValueChange = { pollAnswers[index] = it },
                                    label = { Text("Choice ${index + 1}") },
                                    modifier = Modifier.weight(1f)
                                )
                                if (pollAnswers.size > 2) {
                                    IconButton(onClick = { pollAnswers.removeAt(index) }) {
                                        Icon(Icons.Filled.Close, null)
                                    }
                                }
                            }
                        }
                        if (pollAnswers.size < 10) {
                            item {
                                TextButton(onClick = { pollAnswers.add("") }) {
                                    Icon(Icons.Filled.Add, null)
                                    Spacer(Modifier.width(8.dp))
                                    Text("Add Choice")
                                }
                            }
                        }
                        item {
                            Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
                                Checkbox(checked = pollAllowMultiselect, onCheckedChange = { pollAllowMultiselect = it })
                                Text("Allow multiple answers", style = MaterialTheme.typography.bodyMedium)
                            }
                        }
                        item {
                            val canPost = pollQuestion.isNotBlank() && pollAnswers.count { it.isNotBlank() } >= 2
                            Button(
                                onClick = { 
                                    val poll = Poll(
                                        question = PollMedia(text = pollQuestion),
                                        answers = pollAnswers.filter { it.isNotBlank() }.mapIndexed { idx, text ->
                                            PollAnswer(answer_id = idx + 1, poll_media = PollMedia(text = text))
                                        },
                                        allow_multiselect = pollAllowMultiselect
                                    )
                                    messageStore.sendMessageDraft("", poll = poll)
                                    onDismiss()
                                },
                                enabled = canPost,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(12.dp)
                            ) {
                                Text("Post Poll")
                            }
                        }
                    }
                }
            }

            if (!isMobile && onSystemPickerClick != null) {
                // Material 3 Expressive Floating Toolbar (Desktop)
                MediaPickerTabs(
                    selectedTab = selectedTab,
                    onTabSelected = onTabSelected,
                    onSystemPickerClick = onSystemPickerClick,
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = 16.dp)
                        .zIndex(1f)
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun MediaPickerTabs(
    selectedTab: Int,
    onTabSelected: (Int) -> Unit,
    onSystemPickerClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    HorizontalFloatingToolbar(
        expanded = true,
        modifier = modifier,
        floatingActionButton = {
            FloatingToolbarDefaults.StandardFloatingActionButton(
                onClick = onSystemPickerClick
            ) {
                Icon(Icons.Filled.OpenInNew, "System File Picker")
            }
        },
        colors = FloatingToolbarDefaults.standardFloatingToolbarColors(),
        content = {
            PickerTabItem(
                icon = Icons.Rounded.Image,
                label = "Images",
                isSelected = selectedTab == 0,
                onClick = { onTabSelected(0) }
            )
            PickerTabItem(
                icon = Icons.Filled.Description,
                label = "Files",
                isSelected = selectedTab == 1,
                onClick = { onTabSelected(1) }
            )
            PickerTabItem(
                icon = Icons.Rounded.Devices,
                label = "Camera",
                isSelected = selectedTab == 2,
                onClick = { onTabSelected(2) }
            )
            PickerTabItem(
                icon = Icons.Filled.Tag,
                label = "Thread",
                isSelected = selectedTab == 3,
                onClick = { onTabSelected(3) }
            )
            PickerTabItem(
                icon = Icons.Filled.BarChart,
                label = "Poll",
                isSelected = selectedTab == 4,
                onClick = { onTabSelected(4) }
            )
        }
    )
}

private fun formatDuration(durationMs: Long): String {
    val seconds = (durationMs / 1000) % 60
    val minutes = (durationMs / (1000 * 60)) % 60
    val hours = (durationMs / (1000 * 60 * 60))
    
    return if (hours > 0) {
        "${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}"
    } else {
        "${minutes}:${seconds.toString().padStart(2, '0')}"
    }
}

@Composable
private fun FileItem(
    file: LocalMedia,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    val fileIcon = remember(file.name) {
        when {
            file.name.endsWith(".apk", true) -> Icons.Filled.Android
            file.name.endsWith(".txt", true) -> Icons.Filled.Description
            file.name.endsWith(".pdf", true) -> Icons.Filled.PictureAsPdf
            file.name.endsWith(".zip", true) || file.name.endsWith(".rar", true) -> Icons.Filled.FolderZip
            else -> Icons.Filled.Description
        }
    }

    Surface(
        onClick = onClick,
        color = if (isSelected) MaterialTheme.colorScheme.primaryContainer else Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .background(MaterialTheme.colorScheme.surfaceVariant, RoundedCornerShape(8.dp)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = fileIcon,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            Spacer(Modifier.width(16.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = file.name,
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = formatFileSize(file.size),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            if (isSelected) {
                Icon(
                    imageVector = Icons.Filled.Check,
                    contentDescription = "Selected",
                    tint = MaterialTheme.colorScheme.primary
                )
            }
        }
    }
}

private fun formatFileSize(size: Long): String {
    val units = listOf("B", "KB", "MB", "GB")
    var value = size.toDouble()
    var unitIndex = 0
    while (value >= 1024 && unitIndex < units.size - 1) {
        value /= 1024
        unitIndex++
    }
    return "${value.toInt()} ${units[unitIndex]}"
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun PickerTabItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    val color = if (isSelected) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
    val backgroundColor = if (isSelected) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent
    
    Box(
        modifier = Modifier
            .size(width = 48.dp, height = 40.dp)
            .clip(CircleShape)
            .background(backgroundColor)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = label,
            tint = color,
            modifier = Modifier.size(24.dp)
        )
    }
}
