package me.lampu.lampcord

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import me.lampu.lampcord.shared.settings.Settings

class NetworkChangeReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return
        val activeNetwork = cm.activeNetworkInfo
        val isConnected = activeNetwork != null && activeNetwork.isConnected

        if (isConnected) {
            val token = Settings.shared.discordToken
            val notificationsEnabled = Settings.shared.notificationsEnabled
            if (token.isNotBlank() && notificationsEnabled) {
                GatewayForegroundService.start(context)
            }
        }
    }
}
