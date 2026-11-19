package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable

@Composable
actual fun PlatformRootBackHandler(
    enabled: Boolean,
    onBack: () -> Boolean,
    onExit: () -> Unit,
) {
}
