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
import androidx.compose.foundation.layout.fillMaxSize

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
    
    LaunchedEffect(selectedIndex) {
        if (selectedIndex >= 0 && selectedIndex < items.size) {
            if (!Settings.shared.reduceMotion) scrollState.animateScrollToItem(selectedIndex) else scrollState.scrollToItem(selectedIndex)
        }
    }

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .heightIn(max = 320.dp),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        tonalElevation = 6.dp,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f))
    ) {
        Column {
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                val headerTitle = when(type) {
                    AutocompleteType.MENTION -> "Members & Roles"
                    AutocompleteType.USER -> "Members"
                    AutocompleteType.CHANNEL -> "Channels"
                    AutocompleteType.COMMAND -> "Commands"
                    AutocompleteType.EMOJI -> "Emojis"
                    AutocompleteType.ROLE -> ""
                }
                
                Text(
                    text = headerTitle.uppercase(),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    fontWeight = FontWeight.Black,
                    letterSpacing = 0.5.sp
                )

                if (query.isNotEmpty()) {
                    Text(
                        text = "Matching \"$query\"",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.primary,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
            
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.15f))

            LazyColumn(
                state = scrollState,
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(6.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                itemsIndexed(items, key = { _, item -> item.id }) { index, item ->
                    val isSelected = index == selectedIndex
                    Surface(
                        onClick = { onItemSelected(item) },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(8.dp),
                        color = if (isSelected) MaterialTheme.colorScheme.surfaceContainerHigh else Color.Transparent
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Box(
                                modifier = Modifier.size(if (type == AutocompleteType.EMOJI) 32.dp else 28.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                if (item.icon != null) {
                                    val imageShape = when(type) {
                                        AutocompleteType.MENTION, AutocompleteType.USER -> CircleShape
                                        AutocompleteType.EMOJI -> androidx.compose.ui.graphics.RectangleShape
                                        else -> RoundedCornerShape(4.dp)
                                    }
                                    AsyncImage(
                                        model = item.icon,
                                        contentDescription = null,
                                        modifier = Modifier.fillMaxSize().clip(imageShape),
                                        contentScale = if (type == AutocompleteType.EMOJI) ContentScale.Fit else ContentScale.Crop
                                    )
                                } else if (item.iconType != null) {
                                    Icon(
                                        imageVector = item.iconType,
                                        contentDescription = null,
                                        modifier = Modifier.size(22.dp),
                                        tint = item.color ?: MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                } else {
                                    Box(
                                        modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceVariant, CircleShape),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Text(item.title.take(1).uppercase(), style = MaterialTheme.typography.labelMedium)
                                    }
                                }
                            }
                            
                            Spacer(Modifier.width(12.dp))
                            
                            Column(modifier = Modifier.weight(1f)) {
                                UsernameView(
                                    name = item.title,
                                    style = null,
                                    baseStyle = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold),
                                    color = item.color ?: MaterialTheme.colorScheme.onSurface,
                                    roleGradient = item.gradient,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                                if (item.subtitle != null) {
                                    Text(
                                        text = item.subtitle,
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
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
