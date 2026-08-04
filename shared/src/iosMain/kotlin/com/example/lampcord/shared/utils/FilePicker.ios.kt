package com.example.lampcord.shared.utils

import androidx.compose.runtime.Composable

@Composable
actual fun FilePicker(
    show: Boolean,
    onFileSelected: (List<Pair<String, ByteArray>>) -> Unit,
    onDismiss: () -> Unit
) {
    // TODO: iOS implementation
}
