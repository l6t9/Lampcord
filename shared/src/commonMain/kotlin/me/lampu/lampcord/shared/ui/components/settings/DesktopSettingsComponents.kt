package me.lampu.lampcord.shared.ui.components.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
fun DesktopSettingsLayout(
    content: @Composable ColumnScope.() -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 48.dp, vertical = 24.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Column(
            modifier = Modifier.widthIn(max = 850.dp),
            verticalArrangement = Arrangement.spacedBy(48.dp)
        ) {
            content()
        }
    }
}

@Composable
fun DesktopSettingsSection(
    title: String,
    icon: ImageVector,
    content: @Composable ColumnScope.() -> Unit
) {
    Column(verticalArrangement = Arrangement.spacedBy(24.dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = MaterialTheme.colorScheme.onSurface
            )
            Text(
                text = title,
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface
            )
        }
        
        Column(
            verticalArrangement = Arrangement.spacedBy(20.dp)
        ) {
            content()
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun <T> DesktopButtonGroupSelection(
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    iconProvider: ((T, Boolean) -> ImageVector?)? = null,
    labelProvider: (T) -> String
) {
    ButtonGroup(
        modifier = Modifier.height(44.dp),
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
                                containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.9f),
                                contentColor = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        } else {
                            ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.25f),
                                contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        },
                        contentPadding = PaddingValues(horizontal = 16.dp),
                        modifier = Modifier.fillMaxHeight()
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            if (icon != null) {
                                Icon(icon, null, modifier = Modifier.size(18.dp))
                            }
                            Text(labelProvider(option), style = MaterialTheme.typography.labelLarge)
                        }
                    }
                },
                menuContent = { menuState ->
                    DropdownMenuItem(
                        text = { Text(labelProvider(option)) },
                        leadingIcon = iconProvider?.invoke(option, isSelected)?.let { { Icon(it, null) } },
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

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun <T> DesktopLargeButtonGroupSelection(
    options: List<T>,
    selectedOption: T,
    onOptionSelected: (T) -> Unit,
    iconProvider: (T, Boolean) -> ImageVector,
    labelProvider: (T) -> String
) {
    ButtonGroup(
        modifier = Modifier.height(110.dp).width(260.dp),
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
                            Text(labelProvider(option), style = MaterialTheme.typography.labelLarge)
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
