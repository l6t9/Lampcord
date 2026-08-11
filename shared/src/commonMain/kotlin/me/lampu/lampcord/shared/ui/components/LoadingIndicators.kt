package me.lampu.lampcord.shared.ui.components

import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ContainedLoadingIndicator(
    modifier: Modifier = Modifier,
    containerColor: Color = LoadingIndicatorDefaults.containedContainerColor,
    indicatorColor: Color = LoadingIndicatorDefaults.containedIndicatorColor,
) {
    androidx.compose.material3.ContainedLoadingIndicator(
        modifier = modifier,
        containerColor = containerColor,
        indicatorColor = indicatorColor,
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ContainedLoadingIndicator(
    progress: () -> Float,
    modifier: Modifier = Modifier,
    containerColor: Color = LoadingIndicatorDefaults.containedContainerColor,
    indicatorColor: Color = LoadingIndicatorDefaults.containedIndicatorColor,
) {
    androidx.compose.material3.ContainedLoadingIndicator(
        progress = progress,
        modifier = modifier,
        containerColor = containerColor,
        indicatorColor = indicatorColor,
    )
}

