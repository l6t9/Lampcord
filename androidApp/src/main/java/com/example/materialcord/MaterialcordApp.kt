package com.example.materialcord

import android.app.Application
import com.example.materialcord.shared.di.appModule
import com.example.materialcord.shared.settings.initSettings
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.startKoin

class MaterialcordApp : Application() {
    override fun onCreate() {
        super.onCreate()
        initSettings(this)
        startKoin {
            androidContext(this@MaterialcordApp)
            modules(appModule)
        }
    }
}
