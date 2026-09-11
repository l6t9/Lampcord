package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

// TODO: Missing implementation, assume online for now. (NWPathMonitor?)
actual object NetworkMonitor {
    actual val isOnline: StateFlow<Boolean> = MutableStateFlow(true)
}
