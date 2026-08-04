package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable

@Composable
expect fun FilePicker(
    show: Boolean,
    onFileSelected: (List<Pair<String, ByteArray>>) -> Unit,
    onDismiss: () -> Unit
)
