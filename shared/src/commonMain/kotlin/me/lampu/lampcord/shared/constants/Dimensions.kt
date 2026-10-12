/**
 * Metrolist Project (C) 2026
 * Licensed under GPL-3.0 | See git history for contributors
 */

package me.lampu.lampcord.shared.constants

import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

val NavigationBarHeight = 80.dp
val SlimNavBarHeight = 64.dp
val MiniPlayerHeight = 64.dp
val QueuePeekHeight = 64.dp

val ListItemHeight = 64.dp
val ListThumbnailSize = 48.dp

val ThumbnailCornerRadius = 3.dp

val WideNavigationRailExpandedWidth = 220.dp
val WideNavigationRailCollapsedWidth = 96.dp

val NavigationBarAnimationSpec =
    spring<Dp>(
        dampingRatio = Spring.DampingRatioNoBouncy,
        stiffness = Spring.StiffnessLow,
    )
