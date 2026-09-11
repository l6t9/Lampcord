package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.flow.StateFlow

// Platform reachability. False while the OS reports no usable network for this app.
expect object NetworkMonitor {
    val isOnline: StateFlow<Boolean>
}
