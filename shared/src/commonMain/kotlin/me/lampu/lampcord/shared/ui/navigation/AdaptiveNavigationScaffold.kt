package me.lampu.lampcord.shared.ui.navigation

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.spring
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationItemIconPosition
import androidx.compose.material3.ShortNavigationBar
import androidx.compose.material3.ShortNavigationBarArrangement
import androidx.compose.material3.ShortNavigationBarItem
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.WideNavigationRail
import androidx.compose.material3.WideNavigationRailDefaults
import androidx.compose.material3.WideNavigationRailItem
import androidx.compose.material3.WideNavigationRailValue
import androidx.compose.material3.adaptive.currentWindowAdaptiveInfoV2
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteColors
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteDefaults
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteItem
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteScaffold
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteScaffoldValue
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteType
import androidx.compose.material3.adaptive.navigationsuite.rememberNavigationSuiteScaffoldState
import androidx.compose.material3.rememberWideNavigationRailState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.window.core.layout.WindowSizeClass
import me.lampu.lampcord.shared.ui.icons.Icons

private val WideNavigationRailCollapsedWidth = 80.dp
private val WideNavigationRailExpandedWidth = 240.dp

class AdaptiveNavigationItem<T>(
    val key: T,
    val label: String,
    val icon: ImageVector,
    val selectedIcon: ImageVector = icon,
    val visible: Boolean = true,
)

/** Device conventions that cannot be inferred from an app window's dimensions. */
enum class AdaptiveFormFactor {
    /** Phones, foldables, and tablets; the current app window determines the layout. */
    TOUCH,

    DESKTOP,
    TELEVISION,

    /** The parked app UI. Android Auto and driving-mode AAOS use the system MediaLibrary host. */
    AUTOMOTIVE,
}

enum class AdaptiveWidthBreakpoint {
    COMPACT,
    MEDIUM,
    EXPANDED,
    LARGE,
    EXTRA_LARGE,
}

enum class AdaptiveHeightBreakpoint {
    COMPACT,
    MEDIUM,
    EXPANDED,
}

@Immutable
data class AdaptiveLayout(
    val formFactor: AdaptiveFormFactor,
    val widthBreakpoint: AdaptiveWidthBreakpoint,
    val heightBreakpoint: AdaptiveHeightBreakpoint,
    val recommendedPaneCount: Int,
    val maximumPaneCount: Int,
    val navigationSuiteType: NavigationSuiteType,
)

val NavigationSuiteType.isNavigationBar: Boolean
    get() =
        (this == NavigationSuiteType.ShortNavigationBarCompact) ||
            (this == NavigationSuiteType.ShortNavigationBarMedium) ||
            (this == NavigationSuiteType.NavigationBar)

val NavigationSuiteType.isShortNavigationBar: Boolean
    get() =
        (this == NavigationSuiteType.ShortNavigationBarCompact) ||
            (this == NavigationSuiteType.ShortNavigationBarMedium)

@Composable
fun currentAdaptiveLayout(formFactor: AdaptiveFormFactor): AdaptiveLayout {
    val adaptiveInfo = currentWindowAdaptiveInfoV2()
    return calculateAdaptiveLayout(
        formFactor = formFactor,
        windowSizeClass = adaptiveInfo.windowSizeClass,
        isTabletop = adaptiveInfo.windowPosture.isTabletop,
    )
}

internal fun calculateAdaptiveLayout(
    formFactor: AdaptiveFormFactor,
    minWidthDp: Int,
    minHeightDp: Int,
    isTabletop: Boolean = false,
): AdaptiveLayout =
    calculateAdaptiveLayout(
        formFactor = formFactor,
        windowSizeClass = WindowSizeClass(minWidthDp, minHeightDp),
        isTabletop = isTabletop,
    )

internal fun calculateAdaptiveLayout(
    formFactor: AdaptiveFormFactor,
    windowSizeClass: WindowSizeClass,
    isTabletop: Boolean = false,
): AdaptiveLayout {
    val widthBreakpoint =
        when {
            windowSizeClass.isWidthAtLeastBreakpoint(WindowSizeClass.WIDTH_DP_EXTRA_LARGE_LOWER_BOUND) -> {
                AdaptiveWidthBreakpoint.EXTRA_LARGE
            }

            windowSizeClass.isWidthAtLeastBreakpoint(WindowSizeClass.WIDTH_DP_LARGE_LOWER_BOUND) -> {
                AdaptiveWidthBreakpoint.LARGE
            }

            windowSizeClass.isWidthAtLeastBreakpoint(WindowSizeClass.WIDTH_DP_EXPANDED_LOWER_BOUND) -> {
                AdaptiveWidthBreakpoint.EXPANDED
            }

            windowSizeClass.isWidthAtLeastBreakpoint(WindowSizeClass.WIDTH_DP_MEDIUM_LOWER_BOUND) -> {
                AdaptiveWidthBreakpoint.MEDIUM
            }

            else -> {
                AdaptiveWidthBreakpoint.COMPACT
            }
        }
    val heightBreakpoint =
        when {
            windowSizeClass.isHeightAtLeastBreakpoint(WindowSizeClass.HEIGHT_DP_EXPANDED_LOWER_BOUND) -> {
                AdaptiveHeightBreakpoint.EXPANDED
            }

            windowSizeClass.isHeightAtLeastBreakpoint(WindowSizeClass.HEIGHT_DP_MEDIUM_LOWER_BOUND) -> {
                AdaptiveHeightBreakpoint.MEDIUM
            }

            else -> {
                AdaptiveHeightBreakpoint.COMPACT
            }
        }
    val hasRoomForMultiplePanes =
        (heightBreakpoint != AdaptiveHeightBreakpoint.COMPACT) &&
            (widthBreakpoint >= AdaptiveWidthBreakpoint.EXPANDED)
    val recommendedPaneCount = if (hasRoomForMultiplePanes) 2 else 1
    val maximumPaneCount =
        when {
            heightBreakpoint == AdaptiveHeightBreakpoint.COMPACT -> 1
            widthBreakpoint == AdaptiveWidthBreakpoint.EXTRA_LARGE -> 3
            widthBreakpoint >= AdaptiveWidthBreakpoint.MEDIUM -> 2
            else -> 1
        }
    val automaticNavigationType =
        when {
            widthBreakpoint == AdaptiveWidthBreakpoint.COMPACT -> NavigationSuiteType.ShortNavigationBarCompact
            heightBreakpoint == AdaptiveHeightBreakpoint.COMPACT || isTabletop -> NavigationSuiteType.ShortNavigationBarMedium
            else -> NavigationSuiteType.WideNavigationRailCollapsed
        }
    val navigationSuiteType =
        when (formFactor) {
            AdaptiveFormFactor.TELEVISION -> {
                NavigationSuiteType.WideNavigationRailExpanded
            }

            AdaptiveFormFactor.AUTOMOTIVE -> {
                if (!automaticNavigationType.isNavigationBar) {
                    NavigationSuiteType.WideNavigationRailExpanded
                } else {
                    automaticNavigationType
                }
            }

            else -> {
                automaticNavigationType
            }
        }

    return AdaptiveLayout(
        formFactor = formFactor,
        widthBreakpoint = widthBreakpoint,
        heightBreakpoint = heightBreakpoint,
        recommendedPaneCount = recommendedPaneCount,
        maximumPaneCount = maximumPaneCount,
        navigationSuiteType = navigationSuiteType,
    )
}

@Composable
fun <T> AdaptiveNavigationScaffold(
    layout: AdaptiveLayout,
    items: List<AdaptiveNavigationItem<T>>,
    selectedKey: T?,
    onSelect: (T) -> Unit,
    modifier: Modifier = Modifier,
    navigationVisible: Boolean = true,
    navigationSuiteType: NavigationSuiteType = layout.navigationSuiteType,
    navigationSuiteColors: NavigationSuiteColors = NavigationSuiteDefaults.colors(),
    containerColor: Color = Color.Transparent,
    navigationItemVerticalArrangement: Arrangement.Vertical = Arrangement.Top,
    itemModifier: @Composable (T) -> Modifier = { Modifier },
    navigationHeader: @Composable (NavigationSuiteType) -> Unit = {},
    navigationFooter: (@Composable (isExpanded: Boolean) -> Unit)? = null,
    isSidebarExpanded: Boolean = true,
    onToggleSidebar: () -> Unit = {},
    isEditing: Boolean = false,
    onMove: ((Int, Int) -> Unit)? = null,
    onToggleVisibility: ((T) -> Unit)? = null,
    content: @Composable () -> Unit,
) {
    val visibleItems = if (isEditing) items else items.filter { it.visible }

    val navigationItems: @Composable () -> Unit = {
        visibleItems.forEach { item ->
            val selected = item.key == selectedKey
            NavigationSuiteItem(
                selected = selected,
                onClick = { onSelect(item.key) },
                icon = {
                    AnimatedContent(
                        targetState = selected,
                        transitionSpec = {
                            (fadeIn() + scaleIn(initialScale = 0.8f)) togetherWith
                                (fadeOut() + scaleOut(targetScale = 0.8f))
                        },
                        contentKey = { it },
                    ) { isSelected ->
                        Icon(
                            imageVector = if (isSelected) item.selectedIcon else item.icon,
                            contentDescription = item.label,
                        )
                    }
                },
                label = { Text(item.label) },
                modifier = itemModifier(item.key),
                navigationSuiteType = navigationSuiteType,
            )
        }
    }

    val state =
        rememberNavigationSuiteScaffoldState(
            if (navigationVisible) NavigationSuiteScaffoldValue.Visible else NavigationSuiteScaffoldValue.Hidden,
        )
    LaunchedEffect(navigationVisible) {
        state.snapTo(
            if (navigationVisible) NavigationSuiteScaffoldValue.Visible else NavigationSuiteScaffoldValue.Hidden,
        )
    }

    if (layout.formFactor == AdaptiveFormFactor.DESKTOP &&
        (
            navigationSuiteType == NavigationSuiteType.WideNavigationRailExpanded ||
                navigationSuiteType == NavigationSuiteType.WideNavigationRailCollapsed
        )
    ) {
        DesktopWideNavigationScaffold(
            items = items,
            visibleItems = visibleItems,
            selectedKey = selectedKey,
            onSelect = onSelect,
            navigationHeader = navigationHeader,
            navigationFooter = navigationFooter,
            isSidebarExpanded = isSidebarExpanded,
            onToggleSidebar = onToggleSidebar,
            isEditing = isEditing,
            onMove = onMove,
            onToggleVisibility = onToggleVisibility,
            navigationSuiteType = navigationSuiteType,
            modifier = modifier,
            content = content,
        )
    } else if (navigationSuiteType.isShortNavigationBar) {
        SlimNavigationBar(
            items = visibleItems,
            selectedKey = selectedKey,
            onSelect = onSelect,
            navigationVisible = navigationVisible,
            containerColor = containerColor,
            itemModifier = itemModifier,
            navigationSuiteType = navigationSuiteType,
            modifier = modifier,
            content = content,
        )
    } else {
        NavigationSuiteScaffold(
            navigationItems = navigationItems,
            modifier = modifier,
            navigationSuiteType = navigationSuiteType,
            navigationSuiteColors = navigationSuiteColors,
            state = state,
            containerColor = containerColor,
            navigationItemVerticalArrangement = navigationItemVerticalArrangement,
            primaryActionContent = { navigationHeader(navigationSuiteType) },
            content = content,
        )
    }
}

@Composable
private fun <T> DesktopWideNavigationScaffold(
    items: List<AdaptiveNavigationItem<T>>,
    visibleItems: List<AdaptiveNavigationItem<T>>,
    selectedKey: T?,
    onSelect: (T) -> Unit,
    navigationHeader: @Composable (NavigationSuiteType) -> Unit,
    navigationFooter: (@Composable (isExpanded: Boolean) -> Unit)?,
    isSidebarExpanded: Boolean,
    onToggleSidebar: () -> Unit,
    isEditing: Boolean,
    onMove: ((Int, Int) -> Unit)?,
    onToggleVisibility: ((T) -> Unit)?,
    navigationSuiteType: NavigationSuiteType,
    modifier: Modifier,
    content: @Composable () -> Unit,
) {
    val railState =
        rememberWideNavigationRailState(
            initialValue = if (isSidebarExpanded) WideNavigationRailValue.Expanded else WideNavigationRailValue.Collapsed,
        )
    LaunchedEffect(isSidebarExpanded) {
        if (isSidebarExpanded) railState.expand() else railState.collapse()
    }

    Row(modifier = modifier) {
        Surface(
            modifier =
                Modifier
                    .width(if (isSidebarExpanded) WideNavigationRailExpandedWidth else WideNavigationRailCollapsedWidth)
                    .fillMaxHeight(),
            color = MaterialTheme.colorScheme.surfaceContainer,
        ) {
            Column(
                modifier = Modifier.fillMaxSize(),
            ) {
                Box(
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                ) {
                    WideNavigationRail(
                        state = railState,
                        modifier = Modifier.fillMaxSize(),
                        header = {
                            if (isSidebarExpanded) {
                                navigationHeader(NavigationSuiteType.WideNavigationRailExpanded)
                            } else {
                                Box(
                                    modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    IconButton(
                                        onClick = onToggleSidebar,
                                        modifier = Modifier.size(48.dp),
                                    ) {
                                        Icon(
                                            Icons.Filled.Menu,
                                            contentDescription = "Expand Sidebar",
                                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                            modifier = Modifier.size(24.dp),
                                        )
                                    }
                                }
                            }
                        },
                        colors =
                            WideNavigationRailDefaults.colors(
                                containerColor = Color.Transparent,
                            ),
                        contentPadding =
                            PaddingValues(
                                start = 4.dp,
                                end = if (isSidebarExpanded) 4.dp else 0.dp,
                            ),
                    ) {
                        if (isEditing && onToggleVisibility != null) {
                            val lazyListState = rememberLazyListState()
                            LazyColumn(
                                state = lazyListState,
                                modifier = Modifier.fillMaxWidth().fillMaxHeight(),
                            ) {
                                itemsIndexed(items, key = { _, item -> item.key.toString() }) { _, item ->
                                    EditingSidebarNavItem(
                                        item = item,
                                        onToggleVisibility = { onToggleVisibility(item.key) },
                                    )
                                }
                            }
                        } else {
                            visibleItems.forEach { item ->
                                val selected = item.key == selectedKey
                                WideNavigationRailItem(
                                    selected = selected,
                                    onClick = { onSelect(item.key) },
                                    icon = {
                                        Icon(
                                            imageVector = if (selected) item.selectedIcon else item.icon,
                                            contentDescription = item.label,
                                        )
                                    },
                                    label = {
                                        Text(
                                            item.label,
                                            textAlign = if (isSidebarExpanded) TextAlign.Start else TextAlign.Center,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                            modifier = Modifier.fillMaxWidth(),
                                        )
                                    },
                                    railExpanded = isSidebarExpanded,
                                    iconPosition =
                                        if (isSidebarExpanded) {
                                            NavigationItemIconPosition.Start
                                        } else {
                                            NavigationItemIconPosition.Top
                                        },
                                    modifier = Modifier.padding(vertical = 4.dp),
                                )
                            }
                        }
                    }
                }
                if (navigationFooter != null) {
                    Box(
                        modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        navigationFooter(isSidebarExpanded)
                    }
                }
            }
        }
        content()
    }
}

@Composable
private fun <T> SlimNavigationBar(
    items: List<AdaptiveNavigationItem<T>>,
    selectedKey: T?,
    onSelect: (T) -> Unit,
    navigationVisible: Boolean,
    containerColor: Color,
    itemModifier: @Composable (T) -> Modifier,
    navigationSuiteType: NavigationSuiteType,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Column(modifier = modifier) {
        Box(
            modifier = Modifier.weight(1f).fillMaxWidth(),
        ) {
            content()
        }
        AnimatedVisibility(
            visible = navigationVisible,
            enter =
                expandVertically(
                    animationSpec = spring(dampingRatio = 0.9f, stiffness = 700f),
                    expandFrom = Alignment.Bottom,
                ),
            exit =
                shrinkVertically(
                    animationSpec = spring(dampingRatio = 0.9f, stiffness = 700f),
                    shrinkTowards = Alignment.Bottom,
                ),
        ) {
            SlimNavigationBarContent(
                items = items,
                selectedKey = selectedKey,
                onSelect = onSelect,
                containerColor = containerColor,
                itemModifier = itemModifier,
                isCompact = navigationSuiteType == NavigationSuiteType.ShortNavigationBarCompact,
            )
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun <T> SlimNavigationBarContent(
    items: List<AdaptiveNavigationItem<T>>,
    selectedKey: T?,
    onSelect: (T) -> Unit,
    containerColor: Color,
    itemModifier: @Composable (T) -> Modifier,
    isCompact: Boolean,
) {
    ShortNavigationBar(
        containerColor = containerColor,
        arrangement =
            if (isCompact) {
                ShortNavigationBarArrangement.EqualWeight
            } else {
                ShortNavigationBarArrangement.Centered
            },
    ) {
        items.forEach { item ->
            val selected = item.key == selectedKey
            ShortNavigationBarItem(
                selected = selected,
                onClick = { onSelect(item.key) },
                icon = {
                    Icon(
                        imageVector = if (selected) item.selectedIcon else item.icon,
                        contentDescription = item.label,
                    )
                },
                label =
                    if (selected) {
                        { Text(item.label) }
                    } else {
                        null
                    },
                iconPosition =
                    if (isCompact) {
                        NavigationItemIconPosition.Top
                    } else {
                        NavigationItemIconPosition.Start
                    },
                modifier = itemModifier(item.key),
            )
        }
    }
}

@Composable
private fun <T> EditingSidebarNavItem(
    item: AdaptiveNavigationItem<T>,
    onToggleVisibility: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colorScheme = MaterialTheme.colorScheme
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .height(56.dp)
                .padding(start = 36.dp, end = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            item.icon,
            contentDescription = item.label,
            tint = if (item.visible) colorScheme.onSurface else colorScheme.onSurface.copy(alpha = 0.3f),
            modifier = Modifier.size(24.dp),
        )

        Spacer(Modifier.width(12.dp))
        Text(
            item.label,
            style = MaterialTheme.typography.bodyMedium,
            color = if (item.visible) colorScheme.onSurface else colorScheme.onSurface.copy(alpha = 0.3f),
            modifier = Modifier.weight(1f),
        )

        Checkbox(
            checked = item.visible,
            onCheckedChange = { onToggleVisibility() },
            colors = CheckboxDefaults.colors(checkedColor = colorScheme.primary),
        )
    }
}
