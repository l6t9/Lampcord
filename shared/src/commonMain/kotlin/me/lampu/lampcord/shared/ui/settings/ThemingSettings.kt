package me.lampu.lampcord.shared.ui.settings

import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.add
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.generated.resources.delete
import me.lampu.lampcord.shared.generated.resources.edit
import me.lampu.lampcord.shared.generated.resources.save
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.carousel.HorizontalCenteredHeroCarousel
import androidx.compose.material3.carousel.rememberCarouselState
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.LampcordTheme
import me.lampu.lampcord.shared.model.ThemeManifest
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.ThemeStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import org.koin.compose.koinInject
import kotlinx.serialization.json.*
import kotlinx.serialization.encodeToString
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.ui.kit.handCursor

private val themeJsonFormat = Json { prettyPrint = true; ignoreUnknownKeys = true }

private sealed interface ThemeCarouselEntry {
    data object Default : ThemeCarouselEntry
    data class Installed(val theme: LampcordTheme) : ThemeCarouselEntry
    data object Add : ThemeCarouselEntry
}

@Composable
fun ThemeSettings(
    onNavigateToEditor: (String) -> Unit,
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject(),
) {
    SettingsSubScreen(
        title = "Theme",
        onNavigateBack = onBack,
    ) {
        ThemeSettingsContent(
            onNavigateToEditor = onNavigateToEditor,
            settingsStore = settingsStore,
        )
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ThemeSettingsContent(
    onNavigateToEditor: (String) -> Unit,
    themeStore: ThemeStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
) {
    var showImportPicker by remember { mutableStateOf(false) }
    var showAddDialog by remember { mutableStateOf(false) }
    val settings = me.lampu.lampcord.shared.settings.Settings.shared

    fun updateTheme(theme: String) {
        settingsStore.updateUserSettings(UserSettings.Partial(theme = theme))
    }

    FilePicker(
        show = showImportPicker,
        onFileSelected = { files ->
            me.lampu.lampcord.shared.utils.Logging.i("Theme", "File selected: ${files.firstOrNull()?.first}")
            files.firstOrNull()?.let { (_, data) ->
                try {
                    themeStore.installTheme(data.decodeToString())
                } catch (e: Exception) {
                    me.lampu.lampcord.shared.utils.Logging.e("Theme", "Error reading file: ${e.message}")
                }
            }
            showImportPicker = false
        },
        onDismiss = { showImportPicker = false }
    )

    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        ThemeModeControls(settingsStore, ::updateTheme)

        ColorPaletteSettings(settingsStore)

        ThemeCarousel(
            themes = themeStore.availableThemes,
            activeThemeName = themeStore.activeThemes.lastOrNull()?.manifest?.name,
            onSelectDefault = { themeStore.clearThemeSelection() },
            onSelectTheme = { themeStore.selectTheme(it) },
            onAdd = { showAddDialog = true },
            onEdit = { onNavigateToEditor(themeJsonFormat.encodeToString(it)) },
            onDelete = { themeStore.deleteTheme(it) },
        )

        Material3SettingsGroup(
            title = "Transparency Mode",
            items = listOf(
                selectSettingsItem(
                    title = "Visual Transparency",
                    options = me.lampu.lampcord.shared.settings.TransparencyMode.entries.toList(),
                    selectedOption = settings.transparencyMode,
                    onOptionSelected = { settings.transparencyMode = it },
                    optionLabel = { it.name.lowercase().replace('_', ' ').replaceFirstChar { char -> char.uppercase() } },
                )
            )
        )

        Material3SettingsGroup(
            title = "Installed Themes",
            content = {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    if (themeStore.availableThemes.isEmpty()) {
                        Box(Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                            Text("No themes installed", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }

                    themeStore.availableThemes.forEachIndexed { index, theme ->
                        ThemeSettingsItem(
                            theme = theme,
                            isFirst = index == 0,
                            isLast = index == themeStore.availableThemes.lastIndex,
                            isActive = theme.manifest.name == themeStore.activeThemes.lastOrNull()?.manifest?.name,
                            onToggle = { enabled ->
                                if (enabled) themeStore.selectTheme(theme) else themeStore.clearThemeSelection()
                            },
                            onEdit = { onNavigateToEditor(themeJsonFormat.encodeToString(theme)) },
                            onDelete = { themeStore.deleteTheme(theme) }
                        )
                    }
                }
            }
        )
    }

    if (showAddDialog) {
        AlertDialog(
            onDismissRequest = { showAddDialog = false },
            title = { Text("Add Theme") },
            text = { Text("Import a theme file or create a new theme to edit.") },
            confirmButton = {
                TextButton(onClick = {
                    showAddDialog = false
                    showImportPicker = true
                }) {
                    Text("Import")
                }
            },
            dismissButton = {
                TextButton(onClick = {
                    val newTheme = LampcordTheme(
                        manifest = ThemeManifest(name = "New Theme", author = "Me")
                    )
                    themeStore.installTheme(Json.encodeToString(newTheme))
                    showAddDialog = false
                }) {
                    Text("Create New")
                }
            }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ThemeCarousel(
    themes: List<LampcordTheme>,
    activeThemeName: String?,
    onSelectDefault: () -> Unit,
    onSelectTheme: (LampcordTheme) -> Unit,
    onAdd: () -> Unit,
    onEdit: (LampcordTheme) -> Unit,
    onDelete: (LampcordTheme) -> Unit,
) {
    val entries = remember(themes) {
        listOf<ThemeCarouselEntry>(ThemeCarouselEntry.Default) +
            themes.map { ThemeCarouselEntry.Installed(it) } +
            ThemeCarouselEntry.Add
    }
    val selectedIndex = activeThemeName?.let { name ->
        entries.indexOfFirst { it is ThemeCarouselEntry.Installed && it.theme.manifest.name == name }
    }?.takeIf { it >= 0 } ?: 0
    val carouselState = rememberCarouselState(
        initialItem = selectedIndex,
        itemCount = { entries.size },
    )

    LaunchedEffect(selectedIndex) {
        val target = selectedIndex.coerceIn(0, entries.lastIndex)
        if (!me.lampu.lampcord.shared.settings.Settings.shared.reduceMotion) {
            carouselState.animateScrollToItem(target)
        } else {
            carouselState.scrollToItem(target)
        }
    }

    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        HorizontalCenteredHeroCarousel(
            state = carouselState,
            modifier = Modifier
                .fillMaxWidth()
                .height(221.dp)
                .padding(horizontal = 24.dp),
            maxItemWidth = 240.dp,
            itemSpacing = 8.dp,
            contentPadding = PaddingValues(horizontal = 16.dp),
        ) { index ->
            val itemShape = MaterialTheme.shapes.extraLarge
            val selected = selectedIndex == index
            val itemModifier = Modifier
                .fillMaxWidth()
                .height(205.dp)
                .maskClip(itemShape)
                .then(
                    if (selected && entries[index] is ThemeCarouselEntry.Installed) {
                        Modifier.maskBorder(
                            BorderStroke(2.dp, MaterialTheme.colorScheme.primary),
                            itemShape,
                        )
                    } else {
                        Modifier
                    }
                )
            when (val entry = entries[index]) {
                ThemeCarouselEntry.Default -> DefaultThemeCard(
                    modifier = itemModifier,
                    selected = selected,
                    onClick = onSelectDefault,
                )
                is ThemeCarouselEntry.Installed -> InstalledThemeCard(
                    modifier = itemModifier,
                    theme = entry.theme,
                    selected = selected,
                    onClick = { onSelectTheme(entry.theme) },
                    onEdit = { onEdit(entry.theme) },
                    onDelete = { onDelete(entry.theme) },
                )
                ThemeCarouselEntry.Add -> AddThemeCard(
                    modifier = itemModifier.maskBorder(
                        BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
                        itemShape,
                    ),
                    onClick = onAdd,
                )
            }
        }

        val selectedLabel = when (val entry = entries.getOrNull(selectedIndex)) {
            is ThemeCarouselEntry.Installed -> entry.theme.manifest.name
            ThemeCarouselEntry.Add -> "Add Theme"
            else -> "Default"
        }
        Text(
            text = selectedLabel,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.fillMaxWidth(),
            textAlign = TextAlign.Center,
        )
        Text(
            text = if (selectedIndex == 0) "Using the system theme" else "Swipe to preview themes",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.fillMaxWidth(),
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun DefaultThemeCard(
    modifier: Modifier = Modifier,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val container = if (selected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceContainerHigh
    val content = if (selected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurface
    Surface(
        modifier = modifier,
        shape = RectangleShape,
        color = container,
        contentColor = content,
        onClick = onClick,
    ) {
        Column(
            modifier = Modifier.fillMaxSize().padding(16.dp),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Icon(
                Icons.Rounded.AutoAwesome,
                contentDescription = null,
                modifier = Modifier.size(28.dp),
            )
            Column {
                Text("Default", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Text(
                    "Follow your system theme",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun AddThemeCard(
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    Surface(
        modifier = modifier,
        shape = RectangleShape,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        contentColor = MaterialTheme.colorScheme.onSurface,
        onClick = onClick,
    ) {
        Column(
            modifier = Modifier.fillMaxSize().padding(16.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Icon(Icons.Rounded.Add, contentDescription = null, modifier = Modifier.size(32.dp))
            Spacer(Modifier.height(12.dp))
            Text("Add Theme", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            Spacer(Modifier.height(4.dp))
            Text(
                "Create or import your own",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Composable
private fun InstalledThemeCard(
    modifier: Modifier = Modifier,
    theme: LampcordTheme,
    selected: Boolean,
    onClick: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    val fallbackSurface = MaterialTheme.colorScheme.surfaceContainerHigh
    val surface = themeColor(theme, "surface", "background", fallback = fallbackSurface)
    val primary = themeColor(theme, "primary", "accent", fallback = MaterialTheme.colorScheme.primary)
    val secondary = themeColor(theme, "secondary", fallback = MaterialTheme.colorScheme.secondary)
    val tertiary = themeColor(theme, "tertiary", fallback = MaterialTheme.colorScheme.tertiary)
    val content = themeColor(theme, "onSurface", "text", fallback = MaterialTheme.colorScheme.onSurface)
    val backgroundUrl = (theme.background["url"] as? JsonPrimitive)?.contentOrNull

    Surface(
        modifier = modifier,
        shape = RectangleShape,
        color = surface,
        contentColor = content,
    ) {
        Box(modifier = Modifier.fillMaxSize().clickableCursor { onClick() }) {
            if (!backgroundUrl.isNullOrBlank()) {
                AsyncImage(
                    model = backgroundUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                )
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(content.copy(alpha = 0.62f)),
                )
            }

            Column(
                modifier = Modifier.fillMaxSize().padding(12.dp),
                verticalArrangement = Arrangement.SpaceBetween,
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .height(40.dp)
                            .clip(RoundedCornerShape(10.dp))
                            .background(primary),
                    )
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .height(40.dp)
                            .clip(RoundedCornerShape(10.dp))
                            .background(secondary),
                    )
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .height(40.dp)
                            .clip(RoundedCornerShape(10.dp))
                            .background(tertiary),
                    )
                }

                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            theme.manifest.name,
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        theme.manifest.author?.let {
                            Text(
                                "by $it",
                                style = MaterialTheme.typography.bodySmall,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                    if (selected) {
                        Icon(Icons.Rounded.Check, contentDescription = "Selected")
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                ) {
                    IconButton(onClick = onEdit) {
                        Icon(Icons.Rounded.Edit, contentDescription = stringResource(Res.string.edit))
                    }
                    IconButton(onClick = onDelete) {
                        Icon(Icons.Rounded.Delete, contentDescription = stringResource(Res.string.delete))
                    }
                }
            }
        }
    }
}

private fun themeColor(
    theme: LampcordTheme,
    vararg keys: String,
    fallback: Color,
): Color {
    for (map in listOf(theme.colors, theme.simple_colors)) {
        for (key in keys) {
            val raw = (map[key] as? JsonPrimitive)?.contentOrNull ?: continue
            parseThemeColor(raw)?.let { return it }
        }
    }
    return fallback
}

private fun parseThemeColor(value: String): Color? {
    val hex = value.removePrefix("#")
    return when (hex.length) {
        6 -> runCatching { Color(0xFF000000 or hex.toLong(16)) }.getOrNull()
        8 -> runCatching { Color(hex.toLong(16)) }.getOrNull()
        else -> null
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ThemeSettingsItem(
    theme: LampcordTheme,
    isFirst: Boolean,
    isLast: Boolean,
    isActive: Boolean,
    onToggle: (Boolean) -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit
) {
    Material3SettingsItemRow(
        isFirst = isFirst,
        isLast = isLast,
        item = Material3SettingsItem(
            icon = null,
            title = { Text(theme.manifest.name) },
            description = {
                Column {
                    theme.manifest.author?.let { 
                        Text("by $it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) 
                    }
                    Spacer(modifier = Modifier.height(12.dp))
                    ButtonGroup(
                        modifier = Modifier.height(36.dp),
                        overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                        horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
                    ) {
                        customItem(
                            buttonGroupContent = {
                                Button(
                                    onClick = onEdit,
                                    shapes = ButtonDefaults.shapes(
                                        shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                        pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape
                                    ),
                                    colors = ButtonDefaults.buttonColors(
                                        containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f),
                                        contentColor = MaterialTheme.colorScheme.onPrimaryContainer
                                    ),
                                    modifier = Modifier.fillMaxHeight(),
                                    contentPadding = PaddingValues(horizontal = 12.dp)
                                ) {
                                    Icon(Icons.Rounded.Edit, null, modifier = Modifier.size(16.dp))
                                    Spacer(Modifier.width(8.dp))
                                    Text(stringResource(Res.string.edit), style = MaterialTheme.typography.labelMedium)
                                }
                            },
                            menuContent = { menuState ->
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
                                    text = { Text(stringResource(Res.string.edit)) },
                                    leadingIcon = { Icon(Icons.Rounded.Edit, null) },
                                    onClick = { onEdit(); menuState.dismiss() }
                                )
                            }
                        )
                        customItem(
                            buttonGroupContent = {
                                Button(
                                    onClick = onDelete,
                                    shapes = ButtonDefaults.shapes(
                                        shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                        pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape
                                    ),
                                    colors = ButtonDefaults.buttonColors(
                                        containerColor = MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.5f),
                                        contentColor = MaterialTheme.colorScheme.onErrorContainer
                                    ),
                                    modifier = Modifier.fillMaxHeight(),
                                    contentPadding = PaddingValues(horizontal = 12.dp)
                                ) {
                                    Icon(Icons.Rounded.Delete, null, modifier = Modifier.size(16.dp))
                                    Spacer(Modifier.width(8.dp))
                                    Text(stringResource(Res.string.delete), style = MaterialTheme.typography.labelMedium)
                                }
                            },
                            menuContent = { menuState ->
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
                                    text = { Text(stringResource(Res.string.delete)) },
                                    leadingIcon = { Icon(Icons.Rounded.Delete, null) },
                                    onClick = { onDelete(); menuState.dismiss() }
                                )
                            }
                        )
                    }
                }
            },
            trailingContent = {
                ExpressiveSwitch(checked = isActive, onCheckedChange = onToggle)
            },
            onClick = null
        )
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ThemeEditorScreen(
    themeJson: String,
    onBack: () -> Unit,
    themeStore: ThemeStore = koinInject()
) {
    var theme by remember { mutableStateOf(Json.decodeFromString<LampcordTheme>(themeJson)) }
    var activeTab by remember { mutableStateOf(0) }
    
    val tabs = listOf("Manifest", "Colors", "Simple Colors", "Drawables", "Background", "Fonts")

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("Theme Editor", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                        Text(theme.manifest.name, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                    }
                },
                actions = {
                    TextButton(onClick = {
                        themeStore.updateTheme(theme)
                        onBack()
                    }) {
                        Text(stringResource(Res.string.save), fontWeight = FontWeight.Bold)
                    }
                }
            )
        }
    ) { padding ->
        Column(modifier = Modifier.padding(padding).fillMaxSize()) {
            PrimaryScrollableTabRow(
                selectedTabIndex = activeTab,
                edgePadding = 16.dp,
                containerColor = MaterialTheme.colorScheme.surface,
                divider = {}
            ) {
                tabs.forEachIndexed { index, title ->
                    Tab(
                        selected = activeTab == index,
                        onClick = { activeTab = index },
                        text = { Text(title) }
                    )
                }
            }
            
            Box(modifier = Modifier.weight(1f)) {
                when (activeTab) {
                    0 -> ManifestEditor(theme) { theme = theme.copy(manifest = it) }
                    1 -> ColorMapEditor("Colors", theme.colors) { theme = theme.copy(colors = it) }
                    2 -> ColorMapEditor("Simple Colors", theme.simple_colors) { theme = theme.copy(simple_colors = it) }
                    3 -> ColorMapEditor("Drawable Tints", theme.drawable_tints) { theme = theme.copy(drawable_tints = it) }
                    4 -> BackgroundEditor(theme.background) { theme = theme.copy(background = it) }
                    5 -> FontEditor(theme.fonts) { theme = theme.copy(fonts = it) }
                }
            }
        }
    }
}

@Composable
fun ManifestEditor(manifest: LampcordTheme, onUpdate: (ThemeManifest) -> Unit) {
    Column(modifier = Modifier.padding(16.dp).fillMaxSize(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        var name by remember { mutableStateOf(manifest.manifest.name) }
        var author by remember { mutableStateOf(manifest.manifest.author ?: "") }
        
        TextField(
            value = name,
            onValueChange = { name = it; onUpdate(manifest.manifest.copy(name = it)) },
            label = { Text("Name") },
            modifier = Modifier.fillMaxWidth()
        )
        TextField(
            value = author,
            onValueChange = { author = it; onUpdate(manifest.manifest.copy(author = it)) },
            label = { Text("Author") },
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
fun ColorMapEditor(title: String, colors: Map<String, JsonElement>, onUpdate: (Map<String, JsonElement>) -> Unit) {
    var searchQuery by remember { mutableStateOf("") }
    val filteredKeys = remember(colors, searchQuery) {
        colors.keys.filter { it.contains(searchQuery, ignoreCase = true) }.sorted()
    }

    var editingKey by remember { mutableStateOf<String?>(null) }
    var showAddDialog by remember { mutableStateOf(false) }
    
    Column(modifier = Modifier.fillMaxSize()) {
        TextField(
            value = searchQuery,
            onValueChange = { searchQuery = it },
            placeholder = { Text("Search keys...") },
            modifier = Modifier.fillMaxWidth().padding(16.dp),
            leadingIcon = { Icon(Icons.Rounded.Search, null) },
            shape = CircleShape,
            colors = TextFieldDefaults.colors(
                focusedIndicatorColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent
            )
        )
        
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(filteredKeys) { key ->
                val element = colors[key]
                val colorValue = if (element is JsonPrimitive) element.content else element.toString()
                
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f),
                    border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.1f))
                ) {
                    Row(
                        modifier = Modifier.padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(key, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Bold)
                            Text(colorValue, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        
                        Box(
                            modifier = Modifier
                                .size(32.dp)
                                .clip(CircleShape)
                                .background(parseColorSafe(colorValue))
                                .border(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f), CircleShape)
                                .clickableCursor { editingKey = key }
                        )
                        
                        IconButton(onClick = {
                            val newColors = colors.toMutableMap()
                            newColors.remove(key)
                            onUpdate(newColors)
                        }) {
                            Icon(Icons.Rounded.Close, null, modifier = Modifier.size(18.dp))
                        }
                    }
                }
            }
            
            item {
                Button(
                    onClick = { showAddDialog = true },
                    modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                    shape = CircleShape,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary
                    )
                ) {
                    Icon(Icons.Rounded.Add, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("Add Override")
                }
            }
        }
    }

    if (editingKey != null) {
        val key = editingKey!!
        val element = colors[key]
        val colorValue = if (element is JsonPrimitive) element.content else element.toString()
        val initialColor = parseColorSafe(colorValue)

        ColorPickerDialog(
            title = "Edit $key",
            initialColor = initialColor,
            onColorSelected = {
                val newColors = colors.toMutableMap()
                val hex = "#${(it.toArgb().toUInt().toString(16).padStart(8, '0'))}"
                newColors[key] = JsonPrimitive(hex)
                onUpdate(newColors)
                editingKey = null
            },
            onDismiss = { editingKey = null }
        )
    }

    if (showAddDialog) {
        var newKey by remember { mutableStateOf("") }
        androidx.compose.ui.window.Dialog(onDismissRequest = { showAddDialog = false }) {
            Surface(
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surface,
                modifier = Modifier.fillMaxWidth().padding(16.dp)
            ) {
                Column(modifier = Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Text("Add Override", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
                    TextField(
                        value = newKey,
                        onValueChange = { newKey = it },
                        label = { Text("Key") },
                        modifier = Modifier.fillMaxWidth()
                    )
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                        TextButton(onClick = { showAddDialog = false }) { Text(stringResource(Res.string.cancel)) }
                        Button(
                            onClick = {
                                if (newKey.isNotBlank()) {
                                    val newColors = colors.toMutableMap()
                                    newColors[newKey] = JsonPrimitive("#ffffffff")
                                    onUpdate(newColors)
                                    showAddDialog = false
                                    editingKey = newKey
                                }
                            },
                            enabled = newKey.isNotBlank(),
                            shape = CircleShape,
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = MaterialTheme.colorScheme.onPrimary
                            )
                        ) {
                            Text(stringResource(Res.string.add))
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ColorPickerDialog(
    title: String,
    initialColor: Color,
    onColorSelected: (Color) -> Unit,
    onDismiss: () -> Unit
) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(24.dp),
            color = MaterialTheme.colorScheme.surface,
            modifier = Modifier.fillMaxWidth().padding(16.dp)
        ) {
            Column(modifier = Modifier.padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                Spacer(Modifier.height(16.dp))
                me.lampu.lampcord.shared.ui.components.HsvColorPicker(
                    initialColor = initialColor,
                    onColorSelected = onColorSelected,
                    onDismiss = onDismiss
                )
            }
        }
    }
}

@Composable
fun BackgroundEditor(background: Map<String, JsonElement>, onUpdate: (Map<String, JsonElement>) -> Unit) {
    Column(modifier = Modifier.padding(16.dp).fillMaxSize(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        val url = (background["url"] as? JsonPrimitive)?.content ?: ""
        val alpha = (background["overlay_alpha"] as? JsonPrimitive)?.contentOrNull?.toFloatOrNull() ?: 1.0f
        
        TextField(
            value = url,
            onValueChange = { onUpdate(background + ("url" to JsonPrimitive(it))) },
            label = { Text("Background URL") },
            modifier = Modifier.fillMaxWidth()
        )
        
        Column {
            Text("Overlay Alpha: ${(alpha * 100).toInt()}%", style = MaterialTheme.typography.labelMedium)
            Slider(
                value = alpha,
                onValueChange = { onUpdate(background + ("overlay_alpha" to JsonPrimitive(it))) },
                valueRange = 0f..1f
            )
        }
    }
}

@Composable
fun FontEditor(fonts: Map<String, JsonElement>, onUpdate: (Map<String, JsonElement>) -> Unit) {
    Column(modifier = Modifier.padding(16.dp).fillMaxSize(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        val url = (fonts["*"] as? JsonPrimitive)?.content ?: ""
        
        TextField(
            value = url,
            onValueChange = { onUpdate(fonts + ("*" to JsonPrimitive(it))) },
            label = { Text("Font URL (*)") },
            modifier = Modifier.fillMaxWidth()
        )
        Text("Aliucord themes typically use a global font URL under the '*' key.", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

private fun parseColorSafe(value: String): Color {
    return try {
        if (value.startsWith("#")) {
            val hex = value.removePrefix("#")
            if (hex.length == 6) Color(hex.toLong(16) or 0xFF000000)
            else Color(hex.toLong(16))
        } else {
            Color(value.toInt())
        }
    } catch (e: Exception) {
        Color.Transparent
    }
}
