package me.lampu.lampcord.shared.ui.settings

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
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.LampcordTheme
import me.lampu.lampcord.shared.model.ThemeManifest
import me.lampu.lampcord.shared.state.ThemeStore
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.FilePicker
import me.lampu.lampcord.shared.utils.setClipboardText
import org.koin.compose.koinInject
import kotlinx.serialization.json.*
import kotlinx.serialization.encodeToString
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.ui.kit.handCursor

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ThemingSettings(
    onNavigateToEditor: (String) -> Unit,
    onBack: () -> Unit
) {
    SettingsSubScreen(
        title = "Themer",
        onNavigateBack = onBack,
        contentScrollable = false
    ) {
        ThemingSettingsContent(onNavigateToEditor = onNavigateToEditor)
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ThemingSettingsContent(
    onNavigateToEditor: (String) -> Unit,
    themeStore: ThemeStore = koinInject()
) {
    var showImportPicker by remember { mutableStateOf(false) }
    val settings = me.lampu.lampcord.shared.settings.Settings.shared

    FilePicker(
        show = showImportPicker,
        onFileSelected = { files ->
            me.lampu.lampcord.shared.utils.Logging.i("Theme", "File selected: ${files.firstOrNull()?.first}")
            files.firstOrNull()?.let { (_, data) ->
                try {
                    val json = data.decodeToString()
                    themeStore.installTheme(json)
                } catch (e: Exception) {
                    me.lampu.lampcord.shared.utils.Logging.e("Theme", "Error reading file: ${e.message}")
                }
            }
            showImportPicker = false
        },
        onDismiss = { showImportPicker = false }
    )

    Column(modifier = Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(bottom = 80.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            item {
                Material3SettingsGroup(
                    title = "Transparency Mode",
                    items = listOf(
                        Material3SettingsItem(
                            title = { Text("Visual Transparency") },
                            description = {
                                Column(modifier = Modifier.padding(top = 8.dp)) {
                                    SettingsButtonGroup(
                                        options = me.lampu.lampcord.shared.settings.TransparencyMode.entries,
                                        selectedOption = settings.transparencyMode,
                                        onOptionSelected = { settings.transparencyMode = it },
                                        labelProvider = { it.name.lowercase().replace('_', ' ').replaceFirstChar { it.uppercase() } }
                                    )
                                }
                            }
                        )
                    )
                )
            }

            item {
                Material3SettingsGroup(
                    items = listOf(
                        switchSettingsItem(
                            title = "Enable Custom Fonts",
                            description = "Enable support for custom fonts. May be unstable",
                            checked = settings.enableCustomFonts,
                            onCheckedChange = { settings.enableCustomFonts = it }
                        ),
                        switchSettingsItem(
                            title = "Enable Custom Sounds",
                            description = "Enable support for custom sounds",
                            checked = settings.enableCustomSounds,
                            onCheckedChange = { settings.enableCustomSounds = it }
                        )
                    )
                )
            }

            item {
                Material3SettingsGroup(
                    title = "Themes",
                    content = {
                        Column(
                            modifier = Modifier.fillMaxWidth(),
                            verticalArrangement = Arrangement.spacedBy(4.dp)
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
                                    isActive = theme.manifest.name in themeStore.enabledThemeNames,
                                    onToggle = { themeStore.toggleTheme(theme, it) },
                                    onEdit = { onNavigateToEditor(Json { prettyPrint = true; ignoreUnknownKeys = true }.encodeToString(theme)) },
                                    onDelete = { themeStore.deleteTheme(theme) }
                                )
                            }
                        }
                    }
                )
            }
        }

        Surface(
            modifier = Modifier.fillMaxWidth(),
            color = MaterialTheme.colorScheme.surfaceContainer,
            tonalElevation = 4.dp
        ) {
            Row(
                modifier = Modifier.padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Button(
                    onClick = { showImportPicker = true },
                    modifier = Modifier.weight(1f),
                    shape = CircleShape,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary
                    )
                ) {
                    Icon(Icons.Rounded.FolderOpen, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("Import Theme")
                }
                
                Button(
                    onClick = {
                        val newTheme = LampcordTheme(
                            manifest = ThemeManifest(name = "New Theme", author = "Me")
                        )
                        themeStore.installTheme(Json.encodeToString(newTheme))
                    },
                    modifier = Modifier.weight(1f),
                    shape = CircleShape,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary
                    )
                ) {
                    Icon(Icons.Rounded.Add, null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("New Theme")
                }
            }
        }
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
                                    Text("Edit", style = MaterialTheme.typography.labelMedium)
                                }
                            },
                            menuContent = { menuState ->
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
                                    text = { Text("Edit") },
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
                                    Text("Delete", style = MaterialTheme.typography.labelMedium)
                                }
                            },
                            menuContent = { menuState ->
                                DropdownMenuItem(
                                    modifier = Modifier.handCursor(),
                                    text = { Text("Delete") },
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
                        Text("Save", fontWeight = FontWeight.Bold)
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
                        TextButton(onClick = { showAddDialog = false }) { Text("Cancel") }
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
                            Text("Add")
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
