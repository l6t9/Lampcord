package me.lampu.lampcord.shared.ui.components.settings

import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenuGroup
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.DropdownMenuPopup
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons

/**
 * Records where the row was pressed so the selection menu can be anchored to
 * the actual tap instead of a narrow trailing button. This is the Android
 * settings-select behaviour used by Metrolist.
 */
private fun Modifier.trackPressPosition(onPress: (IntOffset) -> Unit): Modifier =
    pointerInput(Unit) {
        awaitEachGesture {
            val down = awaitFirstDown(requireUnconsumed = false)
            onPress(IntOffset(down.position.x.toInt(), down.position.y.toInt()))
        }
    }

/**
 * A single-choice Material 3 Expressive popup menu with connected item shapes
 * and a checkmark on the selected option.
 */
@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun <T> ExpressiveSelectionMenu(
    expanded: Boolean,
    onDismissRequest: () -> Unit,
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    optionLabel: (T) -> String,
    anchorOffset: IntOffset,
    optionIcon: (@Composable (T, Boolean) -> Unit)? = null,
) {
    val windowSize = LocalWindowInfo.current.containerSize
    Box(
        Modifier.layout { measurable, constraints ->
            // The settings column scrolls, so height is unbounded there. Both
            // the menu measurement and the layout size need finite limits;
            // Int.MAX_VALUE is rejected by Compose's 24-bit IntSize.
            val maxWidth = if (constraints.hasBoundedWidth) constraints.maxWidth else windowSize.width
            val maxHeight = if (constraints.hasBoundedHeight) constraints.maxHeight else windowSize.height
            val placeable = measurable.measure(Constraints(maxWidth = maxWidth, maxHeight = maxHeight))
            layout(0, 0) {
                placeable.place(anchorOffset.x, anchorOffset.y)
            }
        }
    ) {
        DropdownMenuPopup(
            expanded = expanded,
            onDismissRequest = onDismissRequest,
        ) {
            DropdownMenuGroup(
                shapes = MenuDefaults.groupShape(0, 1),
                modifier = Modifier
                    .heightIn(max = 480.dp)
                    .verticalScroll(rememberScrollState()),
            ) {
                options.forEachIndexed { index, option ->
                    val shapes = MenuDefaults.itemShape(index, options.size)
                    val selected = option == selectedOption
                    val onClick = {
                        onOptionSelected(option)
                        onDismissRequest()
                    }
                    val leadingIcon: (@Composable () -> Unit)? = optionIcon?.let { icon ->
                        { icon(option, selected) }
                    }

                    DropdownMenuItem(
                        selected = selected,
                        onClick = onClick,
                        text = { Text(optionLabel(option)) },
                        shapes = shapes,
                        leadingIcon = leadingIcon,
                        selectedLeadingIcon = {
                            Icon(
                                imageVector = Icons.Filled.Check,
                                contentDescription = null,
                                modifier = Modifier.size(MenuDefaults.LeadingIconSize),
                            )
                        },
                    )
                }
            }
        }
    }
}

/**
 * Full-width Android-style settings select. The whole row is the anchor and
 * the current value is shown on the left with a chevron, instead of a strip of
 * square connected chips.
 */
@Composable
fun <T> SettingsSelectRow(
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    optionLabel: (T) -> String,
    optionIcon: (@Composable (T, Boolean) -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    var expanded by remember { mutableStateOf(false) }
    var anchorOffset by remember { mutableStateOf(IntOffset.Zero) }

    Box(modifier.fillMaxWidth().trackPressPosition { anchorOffset = it }) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp),
            shape = RoundedCornerShape(20.dp),
            color = MaterialTheme.colorScheme.surfaceContainer,
            onClick = { expanded = true },
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (optionIcon != null) {
                    optionIcon(selectedOption, true)
                    Spacer(Modifier.width(12.dp))
                }
                Text(
                    text = optionLabel(selectedOption),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f),
                )
                Icon(
                    imageVector = Icons.Rounded.KeyboardArrowDown,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.rotate(if (expanded) 180f else 0f),
                )
            }
        }

        ExpressiveSelectionMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            options = options,
            selectedOption = selectedOption,
            onOptionSelected = onOptionSelected,
            optionLabel = optionLabel,
            optionIcon = optionIcon,
            anchorOffset = anchorOffset,
        )
    }
}

/**
 * A connected settings row whose title and current selection share one line.
 * The selected value keeps its icon on the right and opens the same expressive
 * menu as [SettingsSelectRow], matching Metrolist's compact dropdown rows.
 */
@Composable
fun <T> selectSettingsItem(
    title: String,
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    optionLabel: (T) -> String,
    modifier: Modifier = Modifier,
    description: String? = null,
    icon: ImageVector? = null,
    enabled: Boolean = true,
    optionIcon: (@Composable (T, Boolean) -> Unit)? = null,
): Material3SettingsItem {
    var expanded by remember { mutableStateOf(false) }
    var anchorOffset by remember { mutableStateOf(IntOffset.Zero) }

    return Material3SettingsItem(
        icon = icon,
        title = { Text(title) },
        description = description?.let { text -> { Text(text) } },
        enabled = enabled,
        modifier = modifier,
        showChevron = false,
        onClick = { expanded = true },
        trailingContent = {
            Box(Modifier.trackPressPosition { anchorOffset = it }) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    optionIcon?.let { icon ->
                        CompositionLocalProvider(LocalContentColor provides MaterialTheme.colorScheme.primary) {
                            icon(selectedOption, true)
                        }
                    }
                    Text(
                        text = optionLabel(selectedOption),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.primary,
                        textAlign = TextAlign.End,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.widthIn(max = 128.dp),
                    )
                    Icon(
                        imageVector = Icons.Rounded.KeyboardArrowDown,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier
                            .size(18.dp)
                            .rotate(if (expanded) 180f else 0f),
                    )
                }
                ExpressiveSelectionMenu(
                    expanded = expanded,
                    onDismissRequest = { expanded = false },
                    options = options,
                    selectedOption = selectedOption,
                    onOptionSelected = { option ->
                        onOptionSelected(option)
                        expanded = false
                    },
                    optionLabel = optionLabel,
                    optionIcon = optionIcon,
                    anchorOffset = anchorOffset,
                )
            }
        },
    )
}
