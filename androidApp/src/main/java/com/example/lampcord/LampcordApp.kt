package com.example.lampcord

import android.app.Application
import com.example.lampcord.shared.di.appModule
import com.example.lampcord.shared.settings.initSettings
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
