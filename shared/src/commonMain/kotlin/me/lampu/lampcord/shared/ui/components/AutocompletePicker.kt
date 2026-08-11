package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.AutocompleteItem
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.state.AutocompleteStore
import org.koin.compose.koinInject

@Composable
fun AutocompletePicker(
    type: AutocompleteType,
    query: String,
    selectedIndex: Int,
    autocompleteStore: AutocompleteStore = koinInject(),
    modifier: Modifier = Modifier,
    onItemSelected: (AutocompleteItem) -> Unit
) {
    val items = autocompleteStore.autocompleteItems

    if (items.isEmpty()) return

    val scrollState = rememberLazyListState()
    
    LaunchedEffect(selectedIndex) {
        if (selectedIndex >= 0 && selectedIndex < items.size) {
            scrollState.animateScrollToItem(selectedIndex)
        }
    }

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(max = 320.dp),
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        tonalElevation = 3.dp,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
    ) {
        Column {
            val headerTitle = when(type) {
                AutocompleteType.MENTION -> "Members & Roles"
                AutocompleteType.USER -> "Members"
                AutocompleteType.CHANNEL -> "Channels"
                AutocompleteType.COMMAND -> "Commands"
                AutocompleteType.EMOJI -> "Emojis"
                AutocompleteType.ROLE -> ""
            }
            
            Text(
                text = headerTitle,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
            )
            
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.1f))

            LazyColumn(
                state = scrollState,
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(4.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                itemsIndexed(items, key = { _, item -> item.id }) { index, item ->
                    val isSelected = index == selectedIndex
                    Surface(
                        onClick = { onItemSelected(item) },
                        modifier = Modifier.fillMaxWidth(),
                        shape = MaterialTheme.shapes.small,
                        color = if (isSelected) MaterialTheme.colorScheme.surfaceContainerHigh else Color.Transparent
                    ) {
                        Row(
                            modifier = Modifier.padding(8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (item.icon != null) {
                                AsyncImage(
                                    model = item.icon,
                                    contentDescription = null,
                                    modifier = Modifier.size(24.dp).clip(if (type == AutocompleteType.MENTION) CircleShape else MaterialTheme.shapes.extraSmall)
                                )
                            } else if (item.iconType != null) {
                                Icon(
                                    imageVector = item.iconType,
                                    contentDescription = null,
                                    modifier = Modifier.size(20.dp),
                                    tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            } else {
                                Box(
                                    modifier = Modifier.size(24.dp).background(MaterialTheme.colorScheme.surfaceVariant, CircleShape),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Text(item.title.take(1).uppercase(), style = MaterialTheme.typography.labelSmall)
                                }
                            }
                            
                            Spacer(Modifier.width(12.dp))
                            
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    text = item.title,
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = item.color ?: MaterialTheme.colorScheme.onSurface,
                                    fontWeight = FontWeight.SemiBold,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                                if (item.subtitle != null) {
                                    Text(
                                        text = item.subtitle,
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
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
