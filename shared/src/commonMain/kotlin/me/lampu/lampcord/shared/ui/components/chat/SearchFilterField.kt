package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.InputChip
import androidx.compose.material3.InputChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDatePickerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.generated.resources.confirm
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_clear
import me.lampu.lampcord.shared.generated.resources.search_filter_menu_title
import me.lampu.lampcord.shared.model.AutocompleteType
import me.lampu.lampcord.shared.settings.Settings
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
import kotlin.time.Instant

private const val SEARCH_SENTINEL = "\u200B"


data class SearchChip(val key: String, val value: String, val display: String = value)

data class SearchSuggestion(
    val title: String,
    val subtitle: String?,
    val avatar: String?,
    val icon: ImageVector?,
    val onPick: () -> Unit,
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

class SearchInputState(
    private val autocompleteStore: AutocompleteStore,
    private val navigationStore: NavigationStore,
    private val onQueryChange: (String) -> Unit,
    private val onFilterCommitted: () -> Unit,
    private val chipDisplay: (key: String, value: String) -> String = { _, value -> value },
) {
    val chips = mutableStateListOf<SearchChip>()
    var typed by mutableStateOf(TextFieldValue(""))
        private set
    var pendingDateKey by mutableStateOf<String?>(null)
    var selectedChip by mutableStateOf<Int?>(null)
    var fieldFocused by mutableStateOf(false)
    /** When set, the input row should reveal the leading content (e.g. prefilled scope chip) instead of auto-scrolling to the end. */
    var revealLeadingContent by mutableStateOf(false)
        private set
    private var contentRevision by mutableIntStateOf(0)
    var focusRequest by mutableIntStateOf(0)
        private set

    val typedText: String get() = typed.text

    fun requestFocus() { focusRequest++ }

    fun setQuery(query: String) {
        val parsed = parseSearchQuery(query)
        chips.clear()
        parsed.tokens.forEach { chips.add(SearchChip(it.key, it.value, chipDisplay(it.key, it.value))) }
        val text = parsed.text.ifEmpty { if (chips.isNotEmpty()) " " else "" }
        typed = TextFieldValue(text, TextRange(text.length))
        selectedChip = null
        revealLeadingContent = true
        contentRevision++
        refreshAutocomplete()
    }

    fun toQuery(): String {
        val parts = chips.map { "${it.key}:${it.value}" }.toMutableList()
        if (typedText.isNotBlank()) parts.add(typedText.trim())
        return parts.joinToString(" ")
    }

    fun finalQuery(): String {
        commitCompletedWord()
        publish()
        refreshAutocomplete()
        return toQuery()
    }

    fun onTypedChange(next: TextFieldValue) {
        var committed = false
        selectedChip = null
        typed = next
        revealLeadingContent = false
        contentRevision++
        if (next.text.endsWith(" ")) committed = commitCompletedWord()
        openDateIfNeeded()
        publish()
        if (committed) onFilterCommitted()
        refreshAutocomplete()
    }

    fun onBackspaceAtStart(): Boolean {
        if (typedText.isNotEmpty() && typedText != " ") return false
        if (typedText == " " && chips.isNotEmpty()) {
            // allow deleting the padding space first; if becomes empty next time handle chip
            typed = TextFieldValue("", TextRange(0))
            revealLeadingContent = false
            contentRevision++
            publish()
            refreshAutocomplete()
            return false
        }
        if (typedText.isNotEmpty()) return false
        val selected = selectedChip
        return if (selected != null && selected in chips.indices) {
            chips.removeAt(selected)
            selectedChip = null
            revealLeadingContent = false
            contentRevision++
            publish()
            refreshAutocomplete()
            onFilterCommitted()
            true
        } else if (chips.isNotEmpty()) {
            selectedChip = chips.lastIndex
            true
        } else {
            false
        }
    }

    fun pickChip(key: String, value: String, display: String = value) {
        val text = dropActiveWord().let { if (it.isEmpty()) " " else "$it " }
        typed = TextFieldValue(text, TextRange(text.length))
        if (key == "in" || key == "sort") {
            chips.removeAll { it.key == key }
        }
        chips.add(SearchChip(key, value, display))
        selectedChip = null
        pendingDateKey = null
        revealLeadingContent = false
        contentRevision++
        publish()
        refreshAutocomplete()
        onFilterCommitted()
    }

    fun startFilter(key: String) {
        commitCompletedWord()
        val base = typedText
        val text = when {
            base.isBlank() -> "$key:"
            base.endsWith(" ") -> "$base$key:"
            else -> "$base $key:"
        }
        typed = TextFieldValue(text, TextRange(text.length))
        selectedChip = null
        revealLeadingContent = false
        contentRevision++
        requestFocus()
        openDateIfNeeded()
        publish()
        refreshAutocomplete()
    }

    fun cancelDate() {
        pendingDateKey = null
        val text = dropActiveWord()
        typed = TextFieldValue(text, TextRange(text.length))
        revealLeadingContent = false
        contentRevision++
        publish()
        refreshAutocomplete()
    }

    fun removeChip(index: Int) {
        if (index !in chips.indices) return
        chips.removeAt(index)
        selectedChip = null
        if (chips.isNotEmpty() && typedText.isEmpty()) typed = TextFieldValue(" ", TextRange(1))
        revealLeadingContent = false
        contentRevision++
        publish()
        refreshAutocomplete()
        onFilterCommitted()
    }

    fun clearAll() {
        chips.clear()
        typed = TextFieldValue("", TextRange(0))
        selectedChip = null
        revealLeadingContent = false
        contentRevision++
        publish()
        refreshAutocomplete()
        onFilterCommitted()
    }

    fun suggestions(): List<SearchSuggestion> {
        val word = activeWord()
        val colon = word.indexOf(':')
        if (colon <= 0) return emptyList()
        val key = canonicalSearchKey(word.substring(0, colon)) ?: return emptyList()
        val def = searchFilterDef(key) ?: return emptyList()
        val partial = word.substring(colon + 1)
        return when (def.kind) {
            SearchFilterKind.USER, SearchFilterKind.CHANNEL -> autocompleteStore.searchAutocompleteItems.map { item ->
                SearchSuggestion(
                    title = item.title,
                    subtitle = item.subtitle,
                    avatar = item.icon,
                    icon = item.iconType,
                    onPick = { pickChip(key, item.searchReplacement ?: item.replacement, item.title) }
                )
            }
            SearchFilterKind.CHOICE -> def.choices
                .filter { it.startsWith(partial, ignoreCase = true) }
                .map { choice ->
                    SearchSuggestion(
                        title = choice,
                        subtitle = def.label,
                        avatar = null,
                        icon = searchFilterIcon(key),
                        onPick = { pickChip(key, choice) }
                    )
                }
            SearchFilterKind.DATE -> emptyList()
        }
    }

    private fun openDateIfNeeded() {
        val word = activeWord()
        val colon = word.indexOf(':')
        if (colon <= 0 || word.length != colon + 1) return
        val def = canonicalSearchKey(word.substring(0, colon))?.let(::searchFilterDef) ?: return
        if (def.kind == SearchFilterKind.DATE) pendingDateKey = def.key
    }

    private fun publish() {
        onQueryChange(toQuery())
    }

    private fun activeWord(): String = typedText.substringAfterLast(' ')

    private fun dropActiveWord(): String = if (typedText.contains(' ')) typedText.substringBeforeLast(' ') else ""

    private fun commitCompletedWord(): Boolean {
        val trimmed = typedText.trimEnd()
        val word = trimmed.substringAfterLast(' ')
        val colon = word.indexOf(':')
        if (colon <= 0) return false
        val key = canonicalSearchKey(word.substring(0, colon)) ?: return false
        val value = word.substring(colon + 1)
        if (value.isEmpty()) return false
        if (key == "in" || key == "sort") {
            chips.removeAll { it.key == key }
        }
        chips.add(SearchChip(key, value, chipDisplay(key, value)))
        val text = if (trimmed.contains(' ')) trimmed.substringBeforeLast(' ') else " "
        typed = TextFieldValue(text, TextRange(text.length))
        revealLeadingContent = false
        contentRevision++
        return true
    }

    internal fun contentVersion(): Int = contentRevision

    private fun refreshAutocomplete() {
        val word = activeWord()
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
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchInputBar(
    state: SearchInputState,
    externalQuery: String,
    placeholder: String,
    onSearch: () -> Unit,
    modifier: Modifier = Modifier,
    trailing: @Composable () -> Unit = {},
) {
    var focused by remember { mutableStateOf(false) }
    val focusRequester = remember { FocusRequester() }
    val interactionSource = remember { MutableInteractionSource() }
    val scrollState = rememberScrollState()

    LaunchedEffect(externalQuery) {
        if (externalQuery != state.toQuery()) state.setQuery(externalQuery)
    }

    LaunchedEffect(state.contentVersion(), state.revealLeadingContent) {
        if (state.revealLeadingContent) {
            scrollState.scrollTo(0)
        } else {
            scrollState.scrollTo(scrollState.maxValue)
        }
    }

    LaunchedEffect(state.focusRequest) {
        if (state.focusRequest > 0) focusRequester.requestFocus()
    }

    val containerColor = MaterialTheme.colorScheme.surfaceContainerHigh

    Surface(
        modifier = modifier,
        shape = SearchBarDefaults.inputFieldShape,
        color = containerColor,
        tonalElevation = 0.dp,
        shadowElevation = 0.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp)
                .clickable(interactionSource = interactionSource, indication = null) {
                    focusRequester.requestFocus()
                }
                .padding(start = 16.dp, end = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Filled.Search,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(Modifier.width(16.dp))
            Row(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .horizontalScroll(scrollState)
                    .then(
                        if (Settings.shared.reduceMotion) Modifier
                        else Modifier.animateContentSize(animationSpec = spring(dampingRatio = 0.9f, stiffness = 500f))
                    ),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                state.chips.forEachIndexed { index, chip ->
                    SearchChipView(
                        chip = chip,
                        selected = state.selectedChip == index,
                        onRemove = { state.removeChip(index) },
                        onClick = {
                            state.selectedChip = if (state.selectedChip == index) null else index
                            focusRequester.requestFocus()
                        }
                    )
                }
                BasicTextField(
                    value = state.typed,
                    onValueChange = { state.onTypedChange(it) },
                    modifier = Modifier
                        .focusRequester(focusRequester)
                        .widthIn(min = if (state.chips.isEmpty()) 120.dp else 48.dp)
                        .defaultMinSize(minWidth = if (state.chips.isEmpty()) 120.dp else 48.dp)
                        .onPreviewKeyEvent { event ->
                            if (event.type == KeyEventType.KeyDown && event.key == Key.Backspace) {
                                state.onBackspaceAtStart()
                            } else {
                                false
                            }
                        }
                        .onFocusChanged {
                            focused = it.isFocused
                            state.fieldFocused = it.isFocused
                        },
                    textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                    keyboardActions = KeyboardActions(onSearch = { onSearch() }),
                    decorationBox = { innerTextField ->
                        Box(contentAlignment = Alignment.CenterStart) {
                            if (state.typedText.isEmpty() && state.chips.isEmpty()) {
                                Text(
                                    text = placeholder,
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                            innerTextField()
                        }
                    }
                )
            }
            trailing()
        }
    }

    state.pendingDateKey?.let { key ->
        SearchDatePickerDialog(
            title = searchFilterDef(key)?.label ?: key,
            onDismiss = { state.cancelDate() },
            onConfirm = { date ->
                state.pendingDateKey = null
                state.pickChip(key, date)
            }
        )
    }
}

@Composable
private fun SearchChipView(chip: SearchChip, selected: Boolean, onRemove: () -> Unit, onClick: () -> Unit) {
    val containerColor = if (selected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainer
    val contentColor = if (selected) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
    InputChip(
        selected = selected,
        onClick = onClick,
        label = {
            Text(
                text = "${chip.key}: ${chip.display}",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.labelLarge
            )
        },
        modifier = Modifier.heightIn(min = 36.dp),
        shape = MaterialTheme.shapes.medium,
        border = InputChipDefaults.inputChipBorder(
            enabled = true,
            selected = selected,
            borderColor = if (selected) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.6f),
            selectedBorderColor = MaterialTheme.colorScheme.secondary.copy(alpha = 0.8f)
        ),
        colors = InputChipDefaults.inputChipColors(
            containerColor = containerColor,
            labelColor = contentColor,
            selectedContainerColor = MaterialTheme.colorScheme.secondaryContainer,
            selectedLabelColor = MaterialTheme.colorScheme.onSecondaryContainer
        )
    )
}

@Composable
fun SearchFilterButton(state: SearchInputState, modifier: Modifier = Modifier) {
    var showMenu by remember { mutableStateOf(false) }
    Box(modifier = modifier) {
        IconButton(onClick = { showMenu = true }) {
            Icon(
                imageVector = Icons.Rounded.Tune,
                contentDescription = stringResource(Res.string.search_filter_menu_title)
            )
        }
        SearchFilterMenu(
            expanded = showMenu,
            onDismiss = { showMenu = false },
            onPick = { def -> state.startFilter(def.key) }
        )
    }
}

@Composable
fun SearchClearButton(state: SearchInputState, modifier: Modifier = Modifier) {
    if (state.typedText.isNotEmpty() || state.chips.isNotEmpty()) {
        IconButton(onClick = { state.clearAll() }, modifier = modifier) {
            Icon(
                imageVector = Icons.Filled.Close,
                contentDescription = stringResource(Res.string.lampcord_settings_clear),
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
fun SearchFilterActions(state: SearchInputState) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        SearchFilterButton(state, modifier = Modifier.size(40.dp))
        SearchClearButton(state, modifier = Modifier.size(40.dp))
    }
}

@Composable
fun SearchSuggestionList(suggestions: List<SearchSuggestion>, modifier: Modifier = Modifier) {
    LazyColumn(modifier = modifier.fillMaxWidth()) {
        items(suggestions) { suggestion ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { suggestion.onPick() }
                    .padding(horizontal = 16.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                SuggestionLeading(suggestion)
                Spacer(Modifier.width(12.dp))
                Column {
                    Text(text = suggestion.title, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    suggestion.subtitle?.let {
                        Text(
                            text = it,
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

@Composable
private fun SuggestionLeading(suggestion: SearchSuggestion) {
    val avatar = suggestion.avatar
    val icon = suggestion.icon
    if (avatar != null) {
        AsyncImage(
            model = avatar,
            contentDescription = null,
            modifier = Modifier.size(32.dp).clip(CircleShape)
        )
    } else if (icon != null) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            tint = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SearchDatePickerDialog(
    title: String,
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit,
) {
    val pickerState = rememberDatePickerState()
    DatePickerDialog(
        onDismissRequest = onDismiss,
        confirmButton = {
            TextButton(
                enabled = pickerState.selectedDateMillis != null,
                onClick = {
                    val millis = pickerState.selectedDateMillis ?: return@TextButton
                    val date = Instant.fromEpochMilliseconds(millis).toLocalDateTime(TimeZone.UTC).date
                    onConfirm(date.toString())
                }
            ) { Text(stringResource(Res.string.confirm)) }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.cancel)) }
        }
    ) {
        Column {
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall,
                modifier = Modifier.padding(start = 24.dp, top = 16.dp)
            )
            DatePicker(state = pickerState)
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
