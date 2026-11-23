package me.lampu.lampcord.shared.ui.components.sheet

import androidx.compose.runtime.Composable

/** Whether a screen reader is active, used to choose the full-height sheet detent. */
@Composable
expect fun screenReaderActive(): Boolean
