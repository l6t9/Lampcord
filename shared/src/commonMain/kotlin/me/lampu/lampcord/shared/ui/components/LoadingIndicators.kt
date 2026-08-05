package me.lampu.lampcord.shared.ui.components

import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color

/**
 * Wrapper around Material 3's expressive "contained" loading indicator (a filled
 * container with a progress arc), mirroring Metrolist's ContainedLoadingIndicator.
 * Keeping the @OptIn here lets every call site use it without opting in.
 */
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

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun WavyLoadingIndicator(
    modifier: Modifier = Modifier,
    color: Color = WavyProgressIndicatorDefaults.indicatorColor,
    trackColor: Color = WavyProgressIndicatorDefaults.trackColor,
) {
    LinearWavyProgressIndicator(
        modifier = modifier,
        color = color,
        trackColor = trackColor
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun CircularWavyLoadingIndicator(
    modifier: Modifier = Modifier,
    color: Color = WavyProgressIndicatorDefaults.indicatorColor,
    trackColor: Color = WavyProgressIndicatorDefaults.trackColor,
) {
    CircularWavyProgressIndicator(
        modifier = modifier,
        color = color,
        trackColor = trackColor
    )
}
