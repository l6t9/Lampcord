package me.lampu.lampcord.shared.ui.components.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun SettingsLayout(
    content: @Composable ColumnScope.() -> Unit
) {
    BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
        val isCompact = maxWidth < 600.dp
        
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(
                    vertical = if (isCompact) 16.dp else 24.dp
                ),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Column(
                modifier = Modifier.widthIn(max = 850.dp).fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                content()
            }
        }
    }
}

@Composable
fun SettingsSection(
    title: String,
    icon: ImageVector,
    actions: @Composable (RowScope.() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit
) {
    BoxWithConstraints {
        val isCompact = maxWidth < 600.dp
        
        Column(verticalArrangement = Arrangement.spacedBy(if (isCompact) 16.dp else 24.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(if (isCompact) 20.dp else 24.dp),
                    tint = MaterialTheme.colorScheme.onSurface
                )
                Text(
                    text = title,
                    style = if (isCompact) MaterialTheme.typography.titleMedium else MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f)
                )
                if (actions != null) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        actions()
                    }
                }
            }
            
            Surface(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(20.dp),
                color = MaterialTheme.colorScheme.surfaceContainer
            ) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(if (isCompact) 12.dp else 20.dp)
                ) {
                    content()
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)
@Composable
fun <T> SettingsButtonGroup(
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    iconProvider: ((T, Boolean) -> ImageVector?)? = null,
    labelProvider: (T) -> String,
    minimumItemWidth: Dp = 85.dp
) {
    BoxWithConstraints {
        val isCompact = maxWidth < 600.dp

        if (maxWidth / options.size >= minimumItemWidth) { // Use a dropdown list if there isn't enough room per item
            ButtonGroup(
                modifier = Modifier
                    .then(if (isCompact) Modifier.fillMaxWidth() else Modifier)
                    .height(44.dp),
                overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
            ) {
                options.forEachIndexed { index, option ->
                    val isSelected = option == selectedOption
                    customItem(
                        buttonGroupContent = {
                            val shapes = when {
                                options.size == 1 -> ButtonDefaults.shapes()
                                index == 0 -> ButtonDefaults.shapes(
                                    shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                    pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape
                                )

                                index == options.lastIndex -> ButtonDefaults.shapes(
                                    shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                    pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape
                                )

                                else -> ButtonDefaults.shapes(
                                    shape = MaterialTheme.shapes.small,
                                    pressedShape = ButtonGroupDefaults.connectedMiddleButtonPressShape
                                )
                            }

                            val icon = iconProvider?.invoke(option, isSelected)

                            Button(
                                onClick = { onOptionSelected(option) },
                                shapes = shapes,
                                colors = if (isSelected) {
                                    ButtonDefaults.buttonColors(
                                        containerColor = MaterialTheme.colorScheme.primaryContainer.copy(
                                            alpha = 0.9f
                                        ),
                                        contentColor = MaterialTheme.colorScheme.onPrimaryContainer
                                    )
                                } else {
                                    ButtonDefaults.buttonColors(
                                        containerColor = MaterialTheme.colorScheme.surfaceContainerHigh.copy(
                                            alpha = 0.25f
                                        ),
                                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                },
                                contentPadding = PaddingValues(horizontal = if (isCompact) 8.dp else 16.dp),
                                modifier = Modifier
                                    .then(if (isCompact) Modifier.weight(1f) else Modifier)
                                    .fillMaxHeight()
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.Center,
                                    modifier = Modifier.fillMaxWidth()
                                ) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                                    ) {
                                        if (icon != null) {
                                            Icon(icon, null, modifier = Modifier.size(18.dp))
                                        }
                                        Text(
                                            labelProvider(option),
                                            style = MaterialTheme.typography.labelLarge,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    }
                                }
                            }
                        },
                        menuContent = { menuState ->
                            DropdownMenuItem(
                                text = { Text(labelProvider(option)) },
                                leadingIcon = iconProvider?.invoke(option, isSelected)
                                    ?.let { { Icon(it, null) } },
                                onClick = {
                                    onOptionSelected(option)
                                    menuState.dismiss()
                                }
                            )
                        }
                    )
                }
            }
        } else {

            var expanded by remember { mutableStateOf(false) }
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box {
                    OutlinedButton(
                        shape = MaterialTheme.shapes.small,
                        onClick = { expanded = !expanded }
                    ) {
                        Text(labelProvider(selectedOption))
                        Spacer(Modifier.width(8.dp))
                        Icon(Icons.Default.KeyboardArrowDown, contentDescription = "Select an option")
                    }

                    DropdownMenu(
                        expanded = expanded,
                        onDismissRequest = {
                            expanded = false

                        },
                    ) {
                        options.forEachIndexed { index, option ->
                            DropdownMenuItem(
                                text = { Text(labelProvider(option)) },
                                onClick = {
                                    onOptionSelected(option)
                                    expanded = false

                                }
                            )
                        }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
internal fun <T> SettingsButtonGroupCustomIcon(
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    iconProvider: @Composable ((T, Boolean) -> Unit)? = null,
    labelProvider: (T) -> String
) {
    BoxWithConstraints {
        val isCompact = maxWidth < 600.dp
        
        ButtonGroup(
            modifier = Modifier
                .then(if (isCompact) Modifier.fillMaxWidth() else Modifier)
                .height(44.dp),
            overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
            horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
        ) {
            options.forEachIndexed { index, option ->
                val isSelected = option == selectedOption
                customItem(
                    buttonGroupContent = {
                        val shapes = when {
                            options.size == 1 -> ButtonDefaults.shapes()
                            index == 0 -> ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape
                            )
                            index == options.lastIndex -> ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape
                            )
                            else -> ButtonDefaults.shapes(
                                shape = MaterialTheme.shapes.small,
                                pressedShape = ButtonGroupDefaults.connectedMiddleButtonPressShape
                            )
                        }

                        Button(
                            onClick = { onOptionSelected(option) },
                            shapes = shapes,
                            colors = if (isSelected) {
                                ButtonDefaults.buttonColors(
                                    containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.9f),
                                    contentColor = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                            } else {
                                ButtonDefaults.buttonColors(
                                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.25f),
                                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            },
                            contentPadding = PaddingValues(horizontal = if (isCompact) 8.dp else 16.dp),
                            modifier = Modifier
                                .then(if (isCompact) Modifier.weight(1f) else Modifier)
                                .fillMaxHeight()
                        ) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.Center,
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                                ) {
                                    iconProvider?.invoke(option, isSelected)
                                    Text(
                                        labelProvider(option),
                                        style = MaterialTheme.typography.labelLarge,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                        }
                    },
                    menuContent = { menuState ->
                        DropdownMenuItem(
                            text = { Text(labelProvider(option)) },
                            leadingIcon = iconProvider?.let { { it(option, isSelected) } },
                            onClick = {
                                onOptionSelected(option)
                                menuState.dismiss()
                            }
                        )
                    }
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun <T> SettingsLargeButtonGroup(
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    iconProvider: (T, Boolean) -> ImageVector,
    labelProvider: (T) -> String
) {
    BoxWithConstraints {
        val isCompact = maxWidth < 600.dp
        
        ButtonGroup(
            modifier = Modifier
                .height(110.dp)
                .then(if (isCompact) Modifier.fillMaxWidth() else Modifier.width(260.dp)),
            overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
            horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween)
        ) {
            options.forEachIndexed { index, option ->
                val isSelected = option == selectedOption
                customItem(
                    buttonGroupContent = {
                        val shapes = when {
                            index == 0 -> ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape
                            )
                            else -> ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape
                            )
                        }

                        Button(
                            onClick = { onOptionSelected(option) },
                            shapes = shapes,
                            colors = if (isSelected) {
                                ButtonDefaults.buttonColors(
                                    containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.9f),
                                    contentColor = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                            } else {
                                ButtonDefaults.buttonColors(
                                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.3f),
                                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            },
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        ) {
                            Column(
                                horizontalAlignment = Alignment.CenterHorizontally,
                                verticalArrangement = Arrangement.Center
                            ) {
                                Icon(iconProvider(option, isSelected), null, modifier = Modifier.size(32.dp))
                                Spacer(Modifier.height(8.dp))
                                Text(
                                    labelProvider(option),
                                    style = MaterialTheme.typography.labelLarge,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        }
                    },
                    menuContent = { menuState ->
                        DropdownMenuItem(
                            text = { Text(labelProvider(option)) },
                            leadingIcon = { Icon(iconProvider(option, isSelected), null) },
                            onClick = {
                                onOptionSelected(option)
                                menuState.dismiss()
                            }
                        )
                    }
                )
            }
        }
    }
}
