package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.PopupProperties
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.search_filter_menu_title
import me.lampu.lampcord.shared.model.AutocompleteItem
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.state.AutocompleteStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SEARCH_FILTER_DEFS
import me.lampu.lampcord.shared.state.SearchFilterDef
import me.lampu.lampcord.shared.state.SearchFilterKind
import me.lampu.lampcord.shared.state.canonicalSearchKey
import me.lampu.lampcord.shared.state.parseSearchQuery
import me.lampu.lampcord.shared.state.searchFilterDef
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.icons.Icons
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

data class SearchSuggestion(
    val title: String,
    val subtitle: String?,
    val avatar: String?,
    val icon: ImageVector?,
    val insertText: String,
)

fun searchFilterIcon(key: String): ImageVector = when (key) {
    "from" -> Icons.Filled.Person
    "mentions" -> Icons.Rounded.AlternateEmail
    "in" -> Icons.Filled.Tag
    "has" -> Icons.Filled.Link
    "exclude" -> Icons.Filled.Remove
    "authortype" -> Icons.Filled.Person
    "sort" -> Icons.AutoMirrored.Filled.List
    else -> Icons.Filled.History
}

private fun activeWordRange(value: TextFieldValue): Pair<Int, Int> {
    val end = value.selection.end.coerceIn(0, value.text.length)
    if (end == 0) return 0 to 0
    val space = value.text.lastIndexOf(' ', end - 1)
    return (if (space < 0) 0 else space + 1) to end
}

fun buildSearchSuggestions(value: TextFieldValue, userItems: List<AutocompleteItem>): List<SearchSuggestion> {
    val (start, end) = activeWordRange(value)
    val word = value.text.substring(start, end)
    if (word.isEmpty()) return emptyList()
    val colon = word.indexOf(':')
    if (colon < 0) {
        return SEARCH_FILTER_DEFS
            .filter { it.key.startsWith(word.lowercase()) }
            .map { def ->
                SearchSuggestion(
                    title = "${def.key}:",
                    subtitle = def.label,
                    avatar = null,
                    icon = searchFilterIcon(def.key),
                    insertText = "${def.key}:"
                )
            }
    }
    val key = canonicalSearchKey(word.substring(0, colon)) ?: return emptyList()
    val def = searchFilterDef(key) ?: return emptyList()
    val partial = word.substring(colon + 1)
    return when (def.kind) {
        SearchFilterKind.USER, SearchFilterKind.CHANNEL -> userItems.map { item ->
            SearchSuggestion(
                title = item.title,
                subtitle = item.subtitle,
                avatar = item.icon,
                icon = item.iconType,
                insertText = "$key:${item.searchReplacement ?: item.replacement} "
            )
        }
        SearchFilterKind.CHOICE, SearchFilterKind.DATE -> {
            val options = if (def.kind == SearchFilterKind.DATE) listOf("today", "yesterday") else def.choices
            options.filter { it.startsWith(partial, ignoreCase = true) }.map { option ->
                SearchSuggestion(
                    title = option,
                    subtitle = def.label,
                    avatar = null,
                    icon = searchFilterIcon(key),
                    insertText = "$key:$option "
                )
            }
        }
    }
}

fun updateSearchAutocomplete(
    value: TextFieldValue,
    autocompleteStore: AutocompleteStore,
    navigationStore: NavigationStore
) {
    val (start, end) = activeWordRange(value)
    val word = value.text.substring(start, end)
    val colon = word.indexOf(':')
    val def = if (colon > 0) canonicalSearchKey(word.substring(0, colon))?.let(::searchFilterDef) else null
    when (def?.kind) {
        SearchFilterKind.USER -> autocompleteStore.updateAutocomplete(
            AutocompleteType.USER,
            word.substring(colon + 1),
            navigationStore.selectedGuild,
            navigationStore.selectedChannel,
            isSearch = true
        )
        SearchFilterKind.CHANNEL -> autocompleteStore.updateAutocomplete(
            AutocompleteType.CHANNEL,
            word.substring(colon + 1),
            navigationStore.selectedGuild,
            navigationStore.selectedChannel,
            isSearch = true
        )
        else -> autocompleteStore.clear(isSearch = true)
    }
}

fun applyTokenAwareDelete(old: TextFieldValue, new: TextFieldValue): TextFieldValue {
    if (new.text.length != old.text.length - 1) return new
    var deleted = 0
    while (deleted < new.text.length && old.text[deleted] == new.text[deleted]) deleted++
    val token = parseSearchQuery(old.text).tokens.firstOrNull { deleted == it.end - 1 } ?: return new
    var start = token.start
    var end = token.end
    if (end < old.text.length && old.text[end] == ' ') {
        end++
    } else if (start > 0 && old.text[start - 1] == ' ') {
        start--
    }
    val result = old.text.removeRange(start, end)
    return TextFieldValue(result, TextRange(start.coerceAtMost(result.length)))
}

fun appendSearchFilter(current: TextFieldValue, key: String): TextFieldValue {
    val prefix = "$key:"
    val base = current.text
    val newText = when {
        base.isEmpty() -> prefix
        base.endsWith(" ") -> base + prefix
        else -> "$base $prefix"
    }
    return TextFieldValue(newText, TextRange(newText.length))
}

private class SearchChipTransformation(
    private val background: Color,
    private val foreground: Color,
) : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val builder = AnnotatedString.Builder(text)
        parseSearchQuery(text.text).tokens.forEach { token ->
            builder.addStyle(
                SpanStyle(background = background, color = foreground, fontWeight = FontWeight.Medium),
                token.start,
                token.end
            )
        }
        return TransformedText(builder.toAnnotatedString(), OffsetMapping.Identity)
    }
}

@Composable
fun SearchFilterField(
    value: TextFieldValue,
    onValueChange: (TextFieldValue) -> Unit,
    placeholder: String,
    onSearch: () -> Unit,
    modifier: Modifier = Modifier,
    onFocusChange: (Boolean) -> Unit = {},
    autocompleteStore: AutocompleteStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
) {
    var focused by remember { mutableStateOf(false) }
    val suggestions = buildSearchSuggestions(value, autocompleteStore.searchAutocompleteItems.toList())
    val chipBackground = MaterialTheme.colorScheme.primaryContainer
    val chipForeground = MaterialTheme.colorScheme.onPrimaryContainer
    val transformation = remember(chipBackground, chipForeground) {
        SearchChipTransformation(chipBackground, chipForeground)
    }

    fun pick(suggestion: SearchSuggestion) {
        val (start, end) = activeWordRange(value)
        val newText = value.text.replaceRange(start, end, suggestion.insertText)
        val newValue = TextFieldValue(newText, TextRange(start + suggestion.insertText.length))
        onValueChange(newValue)
        updateSearchAutocomplete(newValue, autocompleteStore, navigationStore)
    }

    Box(modifier = modifier) {
        BasicTextField(
            value = value,
            onValueChange = { newValue ->
                val updated = applyTokenAwareDelete(value, newValue)
                onValueChange(updated)
                updateSearchAutocomplete(updated, autocompleteStore, navigationStore)
            },
            modifier = Modifier
                .fillMaxWidth()
                .onFocusChanged {
                    focused = it.isFocused
                    onFocusChange(it.isFocused)
                },
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            singleLine = true,
            visualTransformation = transformation,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
            keyboardActions = KeyboardActions(onSearch = { onSearch() }),
            decorationBox = { innerTextField ->
                Box(contentAlignment = Alignment.CenterStart) {
                    if (value.text.isEmpty()) {
                        Text(
                            text = placeholder,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                    }
                    innerTextField()
                }
            }
        )

        DropdownMenu(
            expanded = focused && suggestions.isNotEmpty(),
            onDismissRequest = {},
            properties = PopupProperties(focusable = false),
            modifier = Modifier
                .width(300.dp)
                .background(MaterialTheme.colorScheme.surfaceContainerHigh),
            offset = DpOffset(0.dp, 4.dp)
        ) {
            suggestions.take(12).forEach { suggestion ->
                DropdownMenuItem(
                    text = {
                        Column {
                            Text(
                                text = suggestion.title,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            val subtitleText = suggestion.subtitle
                            if (subtitleText != null) {
                                Text(
                                    text = subtitleText,
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        }
                    },
                    leadingIcon = {
                        val avatar = suggestion.avatar
                        val icon = suggestion.icon
                        if (avatar != null) {
                            AsyncImage(
                                model = avatar,
                                contentDescription = null,
                                modifier = Modifier.size(24.dp).clip(CircleShape)
                            )
                        } else if (icon != null) {
                            Icon(imageVector = icon, contentDescription = null, modifier = Modifier.size(20.dp))
                        }
                    },
                    onClick = { pick(suggestion) }
                )
            }
        }
    }
}

@Composable
fun SearchFilterMenu(
    expanded: Boolean,
    onDismiss: () -> Unit,
    onPick: (SearchFilterDef) -> Unit,
) {
    DropdownMenu(expanded = expanded, onDismissRequest = onDismiss) {
        Text(
            text = stringResource(Res.string.search_filter_menu_title),
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )
        SEARCH_FILTER_DEFS.forEach { def ->
            DropdownMenuItem(
                text = { Text(def.label) },
                leadingIcon = {
                    Icon(
                        imageVector = searchFilterIcon(def.key),
                        contentDescription = null,
                        modifier = Modifier.size(20.dp)
                    )
                },
                onClick = {
                    onDismiss()
                    onPick(def)
                }
            )
        }
    }
}
