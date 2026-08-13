package me.lampu.lampcord

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import me.lampu.lampcord.shared.settings.Settings

class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action
        if (action == Intent.ACTION_BOOT_COMPLETED ||
            action == Intent.ACTION_MY_PACKAGE_REPLACED ||
            action == "android.intent.action.QUICKBOOT_POWERON" ||
            action == Intent.ACTION_LOCKED_BOOT_COMPLETED
        ) {
            val token = Settings.shared.discordToken
            val notificationsEnabled = Settings.shared.notificationsEnabled
            val autoStart = Settings.shared.autoStartOnBoot

            if (token.isNotBlank() && notificationsEnabled && autoStart) {
                GatewayForegroundService.start(context)
            }
        }
    }
}
