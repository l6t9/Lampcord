package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.ScrollState
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
}

@Composable
actual fun VerticalScrollbar(
    state: ScrollState,
    modifier: Modifier,
    isVisible: Boolean
) {
}
