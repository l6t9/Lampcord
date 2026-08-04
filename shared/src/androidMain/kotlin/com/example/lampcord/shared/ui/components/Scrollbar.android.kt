package com.example.lampcord.shared.ui.components

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
actual fun VerticalScrollbar(
    state: LazyListState,
    modifier: Modifier,
    isVisible: Boolean,
    reverseLayout: Boolean
) {
    // Android has native scrollbars if enabled, no-op for custom component
}
