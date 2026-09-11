package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

// No nice way to check for connectivity on desktop, always assume online.
actual object NetworkMonitor {
    actual val isOnline: StateFlow<Boolean> = MutableStateFlow(true)
}
