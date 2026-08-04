package me.lampu.lampcord

import android.app.Application
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.settings.initSettings
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.startKoin

class LampcordApp : Application() {
    override fun onCreate() {
        super.onCreate()
        initSettings(this)
        startKoin {
            androidContext(this@LampcordApp)
            modules(appModule)
        }
    }
}
