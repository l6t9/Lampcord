package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.DateTimeUtils
import me.lampu.lampcord.shared.settings.Settings
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DebugLogScreen(onBack: () -> Unit) {
    val logs by Logging.logs.collectAsState()
    var filterLevel by remember { mutableStateOf<String?>(null) }
    var filterTag by remember { mutableStateOf<String?>(null) }
    var searchQuery by remember { mutableStateOf("") }
    var autoScroll by remember { mutableStateOf(true) }
    
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()

    val filteredLogs = remember(logs, filterLevel, filterTag, searchQuery) {
        logs.filter { entry ->
            (filterLevel == null || entry.level == filterLevel) &&
            (filterTag == null || entry.tag == filterTag) &&
            (searchQuery.isBlank() || entry.message.contains(searchQuery, ignoreCase = true) || entry.tag.contains(searchQuery, ignoreCase = true))
        }
    }

    LaunchedEffect(filteredLogs.size, autoScroll) {
        if (autoScroll && filteredLogs.isNotEmpty()) {
            if (Settings.shared.reduceMotion) listState.scrollToItem(filteredLogs.size - 1) else listState.animateScrollToItem(filteredLogs.size - 1)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Debug Logs") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Rounded.ArrowBack, "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { /* Redundant with StateFlow but kept for UI */ }) {
                        Icon(Icons.Rounded.Refresh, "Refresh")
                    }
                    IconButton(onClick = { Logging.clear() }) {
                        Icon(Icons.Rounded.Delete, "Clear")
                    }
                }
            )
        }
    ) { padding ->
        Column(modifier = Modifier.fillMaxSize().padding(padding)) {
            // Filters
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = { searchQuery = it },
                    placeholder = { Text("Search logs...") },
                    modifier = Modifier.weight(1f),
                    singleLine = true,
                    leadingIcon = { Icon(Icons.Rounded.Search, null) },
                    trailingIcon = if (searchQuery.isNotEmpty()) {
                        { IconButton(onClick = { searchQuery = "" }) { Icon(Icons.Rounded.Close, null) } }
                    } else null
                )
                
                FilterChip(
                    selected = autoScroll,
                    onClick = { autoScroll = !autoScroll },
                    label = { Text("Auto-scroll") }
                )
            }

            // Categories (Levels)
            LazyRow(
                contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                val levels = listOf("D", "I", "W", "E", "WTF")
                item {
                    FilterChip(
                        selected = filterLevel == null,
                        onClick = { filterLevel = null },
                        label = { Text("All") }
                    )
                }
                items(levels) { level ->
                    FilterChip(
                        selected = filterLevel == level,
                        onClick = { filterLevel = if (filterLevel == level) null else level },
                        label = { Text(level) },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = when(level) {
                                "D" -> Color.Gray
                                "I" -> Color.Blue
                                "W" -> Color(0xFFFFA500)
                                "E" -> Color.Red
                                "WTF" -> Color.Magenta
                                else -> MaterialTheme.colorScheme.primaryContainer
                            }
                        )
                    )
                }
            }

            // Tags
            val tags = remember(logs) { logs.map { it.tag }.distinct().sorted() }
            if (tags.isNotEmpty()) {
                LazyRow(
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    item {
                        FilterChip(
                            selected = filterTag == null,
                            onClick = { filterTag = null },
                            label = { Text("All Tags") }
                        )
                    }
                    items(tags) { tag ->
                        FilterChip(
                            selected = filterTag == tag,
                            onClick = { filterTag = if (filterTag == tag) null else tag },
                            label = { Text(tag) }
                        )
                    }
                }
            }

            HorizontalDivider()

            LazyColumn(
                modifier = Modifier.weight(1f).fillMaxWidth(),
                state = listState,
                contentPadding = PaddingValues(8.dp)
            ) {
                items(filteredLogs) { entry ->
                    LogEntryItem(entry)
                }
            }
        }
    }
}

@Composable
fun LogEntryItem(entry: Logging.LogEntry) {
    var expanded by remember { mutableStateOf(false) }
    
    val color = when (entry.level) {
        "D" -> Color.Gray
        "I" -> Color(0xFF2196F3)
        "W" -> Color(0xFFFFA000)
        "E" -> Color(0xFFF44336)
        "WTF" -> Color(0xFFFF00FF)
        else -> MaterialTheme.colorScheme.onSurface
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp)
            .clip(RoundedCornerShape(4.dp))
            .background(color.copy(alpha = 0.05f))
            .clickable { expanded = !expanded }
            .padding(8.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = entry.level,
                color = Color.White,
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier
                    .background(color, RoundedCornerShape(2.dp))
                    .padding(horizontal = 4.dp, vertical = 1.dp)
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = entry.tag,
                style = MaterialTheme.typography.labelSmall,
                color = color,
                fontWeight = FontWeight.Bold
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = DateTimeUtils.formatLogTimestamp(entry.timestamp),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
        
        Spacer(Modifier.height(4.dp))
        
        Text(
            text = entry.message,
            style = MaterialTheme.typography.bodySmall,
            fontFamily = FontFamily.Monospace,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = if (expanded) Int.MAX_VALUE else 3
        )

        if (expanded && entry.throwable != null) {
            Spacer(Modifier.height(8.dp))
            Surface(
                color = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.3f),
                shape = RoundedCornerShape(4.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = entry.throwable,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = MaterialTheme.colorScheme.error,
                    modifier = Modifier.padding(8.dp)
                )
            }
        }
    }
}
