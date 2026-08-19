package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.flow.MutableSharedFlow

data class SnackbarRequest(
    val message: String,
    val actionLabel: String? = null,
    val action: (() -> Unit)? = null
)

object SnackbarManager {
    val flow = MutableSharedFlow<SnackbarRequest>()

    suspend fun show(message: String, actionLabel: String? = null, action: (() -> Unit)? = null) {
        flow.emit(SnackbarRequest(message, actionLabel, action))
    }
}
