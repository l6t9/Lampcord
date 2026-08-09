package me.lampu.lampcord

import android.app.Application
import me.lampu.lampcord.shared.di.appModule
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
            modules(appModule)
        }
    }
}
