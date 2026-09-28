package me.lampu.lampcord.shared.utils

import kotlinx.coroutines.flow.StateFlow

expect object NetworkMonitor {
    val isOnline: StateFlow<Boolean>
}
