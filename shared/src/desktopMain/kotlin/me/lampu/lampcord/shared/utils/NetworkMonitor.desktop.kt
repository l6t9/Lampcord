package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

actual object NetworkMonitor {
    actual val isOnline: StateFlow<Boolean> = MutableStateFlow(true)
}
