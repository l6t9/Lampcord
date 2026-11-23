package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.AutocompleteItem
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.AutocompleteStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private val AutocompleteRowHeight = 48.dp
private val AutocompleteIconSize = 32.dp
private val AutocompleteMaxHeight = 200.dp

@Composable
fun AutocompletePicker(
    type: AutocompleteType,
    query: String,
    selectedIndex: Int,
    isSearch: Boolean = false,
    autocompleteStore: AutocompleteStore = koinInject(),
    modifier: Modifier = Modifier,
    onItemSelected: (AutocompleteItem) -> Unit
) {
    val items = if (isSearch) autocompleteStore.searchAutocompleteItems else autocompleteStore.autocompleteItems
    if (items.isEmpty()) return

    val scrollState = rememberLazyListState()
    val reduceMotion = Settings.shared.reduceMotion

    LaunchedEffect(selectedIndex, items.size) {
        if (selectedIndex in 0 until items.size) {
            if (reduceMotion) {
                scrollState.scrollToItem(selectedIndex)
            } else {
                scrollState.animateScrollToItem(selectedIndex)
            }
        }
    }

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(max = AutocompleteMaxHeight),
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        border = androidx.compose.foundation.BorderStroke(
            1.dp,
            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.45f)
        ),
        tonalElevation = 2.dp
    ) {
        Column {
            Text(
                text = autocompleteSectionTitle(type, query),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
            )

            HorizontalDivider(
                color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f)
            )

            LazyColumn(
                state = scrollState,
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(vertical = 2.dp)
            ) {
                itemsIndexed(items, key = { _, item -> item.id }) { index, item ->
                    if (index > 0) {
                        HorizontalDivider(
                            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f),
                            modifier = Modifier.padding(horizontal = 16.dp)
                        )
                    }
                    AutocompleteResultRow(
                        item = item,
                        type = type,
                        selected = index == selectedIndex,
                        onItemSelected = onItemSelected
                    )
                }
            }
        }
    }
}

@Composable
private fun autocompleteSectionTitle(type: AutocompleteType, query: String): String {
    val title = when (type) {
        AutocompleteType.MENTION -> "Members & roles"
        AutocompleteType.USER -> "Members"
        AutocompleteType.CHANNEL -> "Channels"
        AutocompleteType.COMMAND -> "Commands"
        AutocompleteType.EMOJI -> "Emojis"
        AutocompleteType.ROLE -> "Roles"
    }
    return if (query.isBlank()) title else "$title matching \"$query\""
}

@Composable
private fun AutocompleteResultRow(
    item: AutocompleteItem,
    type: AutocompleteType,
    selected: Boolean,
    onItemSelected: (AutocompleteItem) -> Unit
) {
    val isPerson = type == AutocompleteType.MENTION || type == AutocompleteType.USER
    val isEmoji = type == AutocompleteType.EMOJI
    val isCommand = type == AutocompleteType.COMMAND
    val rowBackground = if (selected) {
        MaterialTheme.colorScheme.primary.copy(alpha = 0.14f)
    } else {
        Color.Transparent
    }

    val description = if (isCommand) item.subtitle else null
    val trailing = when {
        isCommand -> null
        isPerson -> item.subtitle?.takeIf { it != item.title }
        else -> item.subtitle
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(AutocompleteRowHeight)
            .background(rowBackground)
            .clickable(interactionSource = null, indication = null) { onItemSelected(item) }
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(modifier = Modifier.size(AutocompleteIconSize)) {
            when {
                item.icon != null -> {
                    val shape = when {
                        isPerson -> CircleShape
                        isEmoji -> RectangleShape
                        else -> RoundedCornerShape(8.dp)
                    }
                    AsyncImage(
                        model = item.icon,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize().clip(shape),
                        contentScale = if (isEmoji) ContentScale.Fit else ContentScale.Crop
                    )
                }
                item.iconType != null -> {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(
                                MaterialTheme.colorScheme.surfaceContainerHighest,
                                RoundedCornerShape(8.dp)
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = item.iconType,
                            contentDescription = null,
                            modifier = Modifier.size(20.dp),
                            tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                        )
                    }
                }
                isCommand -> {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(
                                MaterialTheme.colorScheme.primaryContainer,
                                RoundedCornerShape(8.dp)
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = "/",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer
                        )
                    }
                }
                else -> {
                    Box(
                        modifier = Modifier.fillMaxSize().background(
                            MaterialTheme.colorScheme.surfaceVariant,
                            CircleShape
                        ),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = item.title.take(1).uppercase(),
                            style = MaterialTheme.typography.labelLarge
                        )
                    }
                }
            }

            if (isPerson && item.status != null) {
                StatusIndicator(
                    status = item.status,
                    size = 10.dp,
                    borderColor = MaterialTheme.colorScheme.surfaceContainerLow,
                    modifier = Modifier.align(Alignment.BottomEnd)
                )
            }
        }

        Spacer(Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (isPerson && (item.color != null || item.gradient != null)) {
                    UsernameView(
                        name = item.title,
                        style = null,
                        baseStyle = MaterialTheme.typography.bodyLarge.copy(
                            fontSize = 16.sp,
                            fontWeight = FontWeight.SemiBold
                        ),
                        color = item.color ?: MaterialTheme.colorScheme.onSurface,
                        roleGradient = item.gradient,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false)
                    )
                } else {
                    Text(
                        text = item.title,
                        style = MaterialTheme.typography.bodyLarge.copy(
                            fontSize = 16.sp,
                            fontWeight = FontWeight.SemiBold
                        ),
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false)
                    )
                }
                if (item.isBot) {
                    Spacer(Modifier.width(6.dp))
                    Surface(
                        shape = RoundedCornerShape(4.dp),
                        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.18f)
                    ) {
                        Text(
                            text = "BOT",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.padding(horizontal = 4.dp, vertical = 1.dp)
                        )
                    }
                }
            }
            if (description != null) {
                Text(
                    text = description,
                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 14.sp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }

        if (trailing != null) {
            Spacer(Modifier.width(12.dp))
            Text(
                text = trailing,
                style = MaterialTheme.typography.bodySmall.copy(fontSize = 14.sp),
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.75f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.heightIn(max = AutocompleteRowHeight)
            )
        }
    }
}
