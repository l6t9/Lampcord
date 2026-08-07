package me.lampu.lampcord.shared.ui.components.settings

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.*
import androidx.compose.foundation.interaction.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch

@Composable
fun Material3SettingsGroup(
    title: String? = null,
    items: List<Material3SettingsItem> = emptyList(),
    horizontalPadding: Dp = 16.dp,
    content: (@Composable () -> Unit)? = null,
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
    ) {
        title?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Medium),
                color = MaterialTheme.colorScheme.primary,
                textAlign = TextAlign.Start,
                modifier = Modifier.padding(
                    bottom = 12.dp,
                    top = 24.dp,
                    start = 16.dp,
                    end = 32.dp,
                ),
            )
        }

        if (content != null) {
            content()
        } else {
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                items.forEachIndexed { index, item ->
                    Material3SettingsItemRow(
                        item = item,
                        isFirst = index == 0,
                        isLast = index == items.lastIndex,
                        horizontalPadding = horizontalPadding,
                    )
                }
            }
        }
    }
}

@Composable
private fun Material3SettingsItemRow(
    item: Material3SettingsItem,
    isFirst: Boolean = false,
    isLast: Boolean = false,
    horizontalPadding: Dp = 16.dp,
) {
    val cornerRadius = 20.dp
    val reducedRadius = 5.dp

    val shape = RoundedCornerShape(
        topStart = if (isFirst) cornerRadius else reducedRadius,
        topEnd = if (isFirst) cornerRadius else reducedRadius,
        bottomStart = if (isLast) cornerRadius else reducedRadius,
        bottomEnd = if (isLast) cornerRadius else reducedRadius,
    )

    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()

    val backgroundColor by animateColorAsState(
        targetValue = if (isPressed) {
            MaterialTheme.colorScheme.surfaceContainerHigh
        } else {
            MaterialTheme.colorScheme.surfaceContainer
        },
        label = "backgroundColor",
    )

    Column(
        modifier = item.modifier
            .fillMaxWidth()
            .padding(horizontal = horizontalPadding)
            .clip(shape)
            .background(color = backgroundColor)
            .animateContentSize(),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clickable(
                    interactionSource = interactionSource,
                    indication = null,
                    enabled = item.enabled && item.onClick != null,
                    onClick = { item.onClick?.invoke() },
                )
                .padding(horizontal = 16.dp, vertical = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (item.leadingContent != null) {
                item.leadingContent.invoke()
                Spacer(modifier = Modifier.width(16.dp))
            } else if (item.icon != null) {
                Box(
                    modifier = Modifier.size(32.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        item.icon,
                        contentDescription = null,
                        tint = item.iconTint ?: MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(24.dp),
                    )
                }
                Spacer(modifier = Modifier.width(12.dp))
            }

            Column(
                modifier = Modifier.weight(1f),
            ) {
                ProvideTextStyle(
                    MaterialTheme.typography.titleMedium.copy(
                        fontWeight = FontWeight.W500,
                        color = if (!item.enabled) {
                            MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f)
                        } else {
                            MaterialTheme.colorScheme.onSurface
                        },
                    ),
                ) {
                    item.title()
                }

                item.description?.let { desc ->
                    Spacer(modifier = Modifier.height(2.dp))
                    ProvideTextStyle(
                        MaterialTheme.typography.bodySmall.copy(
                            color = if (!item.enabled) {
                                MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.38f)
                            } else {
                                MaterialTheme.colorScheme.onSurfaceVariant
                            },
                        ),
                    ) {
                        desc()
                    }
                }
            }

            if (item.trailingContent != null) {
                Spacer(modifier = Modifier.width(8.dp))
                item.trailingContent.invoke()
            } else if (item.expandableContent != null) {
                val rotation by animateFloatAsState(
                    targetValue = if (item.expanded) 90f else 0f,
                    animationSpec = tween(200)
                )
                Icon(
                    imageVector = Icons.Outlined.ChevronRight,
                    contentDescription = if (item.expanded) "Collapse" else "Expand",
                    modifier = Modifier.graphicsLayer { rotationZ = rotation },
                    tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = if (item.enabled) 1f else 0.38f),
                )
            }
        }

        AnimatedVisibility(visible = item.expanded && item.enabled) {
            item.expandableContent?.invoke()
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsSubScreen(
    title: String,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
    contentScrollable: Boolean = true,
    content: @Composable () -> Unit,
) {
    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior()
    
    Scaffold(
        modifier = modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
        containerColor = MaterialTheme.colorScheme.surface,
        topBar = {
            LargeTopAppBar(
                title = { Text(title) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                scrollBehavior = scrollBehavior
            )
        }
    ) { padding ->
        val scrollState = rememberScrollState()
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .then(if (contentScrollable) Modifier.verticalScroll(scrollState) else Modifier)
                .padding(vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            content()
        }
    }
}

@Composable
fun SettingsExpandableActionRow(
    title: String,
    subtitle: String? = null,
    expanded: Boolean,
    onToggle: () -> Unit,
    enabled: Boolean = true,
    content: @Composable () -> Unit,
) {
    Material3SettingsItemRow(
        item = Material3SettingsItem(
            title = { Text(title) },
            description = subtitle?.let { { Text(it) } },
            expanded = expanded,
            expandableContent = content,
            enabled = enabled,
            onClick = onToggle
        ),
        isFirst = true,
        isLast = true
    )
}

@Composable
fun SettingsActionRow(
    title: String,
    subtitle: String,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    Material3SettingsItemRow(
        item = Material3SettingsItem(
            title = { Text(title) },
            description = { Text(subtitle) },
            enabled = enabled,
            onClick = onClick
        ),
        isFirst = true,
        isLast = true
    )
}

@Composable
fun SettingsToggle(
    title: String,
    subtitle: String? = null,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    enabled: Boolean = true,
) {
    Material3SettingsItemRow(
        item = switchSettingsItem(
            title = title,
            description = subtitle,
            checked = checked,
            enabled = enabled,
            onCheckedChange = onCheckedChange
        ),
        isFirst = true,
        isLast = true
    )
}

data class Material3SettingsItem(
    val icon: ImageVector? = null,
    val leadingContent: (@Composable () -> Unit)? = null,
    val title: @Composable () -> Unit,
    val description: (@Composable () -> Unit)? = null,
    val trailingContent: (@Composable () -> Unit)? = null,
    val enabled: Boolean = true,
    val iconContainerColor: Color? = null,
    val iconTint: Color? = null,
    val modifier: Modifier = Modifier,
    val onClick: (() -> Unit)? = null,
    val expanded: Boolean = false,
    val expandableContent: (@Composable () -> Unit)? = null,
)

@Composable
fun switchSettingsItem(
    title: String,
    description: String? = null,
    checked: Boolean,
    enabled: Boolean = true,
    onCheckedChange: (Boolean) -> Unit
): Material3SettingsItem {
    return Material3SettingsItem(
        title = { Text(title) },
        description = description?.let { { Text(it) } },
        enabled = enabled,
        trailingContent = {
            ExpressiveSwitch(checked = checked, onCheckedChange = onCheckedChange, enabled = enabled)
        },
        onClick = { onCheckedChange(!checked) }
    )
}

fun expandableSettingsItem(
    title: String,
    description: String? = null,
    expanded: Boolean,
    enabled: Boolean = true,
    onToggle: () -> Unit,
    content: @Composable () -> Unit,
): Material3SettingsItem {
    return Material3SettingsItem(
        title = { Text(title) },
        description = description?.let { { Text(it) } },
        expanded = expanded,
        expandableContent = content,
        enabled = enabled,
        onClick = onToggle
    )
}

@Composable
fun navigationSettingsItem(
    title: String,
    description: String? = null,
    enabled: Boolean = true,
    onClick: () -> Unit
): Material3SettingsItem {
    return Material3SettingsItem(
        title = { Text(title) },
        description = description?.let { { Text(it) } },
        enabled = enabled,
        trailingContent = {
            Icon(Icons.AutoMirrored.Filled.ArrowForward, contentDescription = null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
        },
        onClick = onClick
    )
}

@Composable
fun rememberSettingsIconTones(): Map<String, Pair<Color, Color>> {
    val palette = MaterialTheme.colorScheme
    return remember(palette) {
        mapOf(
            "neutral" to Pair(palette.primary, palette.primary.copy(alpha = 0.1f)),
            "orange" to Pair(Color(0xFFFF9800), Color(0xFFFF9800).copy(alpha = 0.1f)),
            "rose" to Pair(Color(0xFFE91E63), Color(0xFFE91E63).copy(alpha = 0.1f)),
            "cyan" to Pair(Color(0xFF00BCD4), Color(0xFF00BCD4).copy(alpha = 0.1f)),
            "gold" to Pair(Color(0xFFFFC107), Color(0xFFFFC107).copy(alpha = 0.1f)),
            "blue" to Pair(Color(0xFF2196F3), Color(0xFF2196F3).copy(alpha = 0.1f)),
            "green" to Pair(Color(0xFF4CAF50), Color(0xFF4CAF50).copy(alpha = 0.1f)),
            "purple" to Pair(Color(0xFF9C27B0), Color(0xFF9C27B0).copy(alpha = 0.1f)),
        )
    }
}
