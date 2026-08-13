package me.lampu.lampcord

import android.app.Application
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.initSettings
import me.lampu.lampcord.shared.utils.AndroidContextProvider
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.startKoin

class LampcordApp : Application() {
    override fun onCreate() {
        super.onCreate()
        AndroidContextProvider.applicationContext = this
        initSettings(this)
        startKoin {
            androidContext(this@LampcordApp)
            modules(appModule, androidNotificationModule)
        }

        NotificationHelper.ensureMessageChannel(this)
        GatewayForegroundService.ensureServiceChannel(this)
        PushRelayManager.registerCurrentDevice()


        AppLifecycleTracker.register(this)
        AppLifecycleTracker.onForegroundChanged = { isInForeground ->
            if (!isInForeground && Settings.shared.discordToken.isNotBlank()) {
                GatewayForegroundService.start(this)
            } else if (isInForeground) {
                GatewayForegroundService.stop(this)
            }
        }
    }
}
