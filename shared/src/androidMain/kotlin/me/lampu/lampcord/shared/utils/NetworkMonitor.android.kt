package me.lampu.lampcord.shared.utils

import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

actual object NetworkMonitor {
    private val state: MutableStateFlow<Boolean> by lazy {
        val manager = AndroidContextProvider.applicationContext.getSystemService(ConnectivityManager::class.java)
            ?: return@lazy MutableStateFlow(true)

        val flow = MutableStateFlow(hasInternet(manager))

        manager.registerDefaultNetworkCallback(object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) { flow.value = hasInternet(manager) }
            override fun onLost(network: Network) { flow.value = hasInternet(manager) }
            override fun onUnavailable() { flow.value = hasInternet(manager) }
            // Doze, battery saver and data saver block the network without losing it.
            override fun onBlockedStatusChanged(network: Network, blocked: Boolean) { flow.value = hasInternet(manager) }
        })

        flow
    }

    actual val isOnline: StateFlow<Boolean> get() = state

    // activeNetwork is null when there is no default network
    private fun hasInternet(manager: ConnectivityManager): Boolean {
        val caps = manager.getNetworkCapabilities(manager.activeNetwork) ?: return false
        return caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
    }
}
