package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable

@Composable
actual fun PlatformBackHandler(enabled: Boolean, onBack: () -> Unit) {
    // No-op for desktop, could handle ESC key if needed
}
