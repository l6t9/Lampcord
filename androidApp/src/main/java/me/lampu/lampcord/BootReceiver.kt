package me.lampu.lampcord

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.UserManager
import me.lampu.lampcord.shared.settings.Settings

class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action
        if (action == Intent.ACTION_BOOT_COMPLETED ||
            action == Intent.ACTION_MY_PACKAGE_REPLACED ||
            action == "android.intent.action.QUICKBOOT_POWERON" ||
            action == Intent.ACTION_USER_UNLOCKED
        ) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                val userManager = context.getSystemService(Context.USER_SERVICE) as? UserManager
                if (userManager != null && !userManager.isUserUnlocked) {
                    // Credential encrypted storage is locked; ignore until user unlocks
                    return
                }
            }

            try {
                val token = Settings.shared.discordToken
                val notificationsEnabled = Settings.shared.notificationsEnabled
                val autoStart = Settings.shared.autoStartOnBoot

                if (token.isNotBlank() && notificationsEnabled && autoStart) {
                    GatewayForegroundService.start(context)
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}
