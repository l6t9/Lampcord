package com.example.materialcord

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.example.materialcord.shared.di.appModule
import com.example.materialcord.shared.ui.App
import org.koin.core.context.startKoin

fun main() {
    startKoin {
        modules(appModule)
    }
    application {
        Window(onCloseRequest = ::exitApplication, title = "Materialcord") {
            App()
        }
    }
}
