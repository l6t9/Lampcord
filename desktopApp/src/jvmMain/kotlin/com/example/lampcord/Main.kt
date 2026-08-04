package com.example.lampcord

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import com.example.lampcord.shared.di.appModule
import com.example.lampcord.shared.ui.App
import com.example.lampcord.utils.WaylandScale
import org.koin.core.context.startKoin

fun main() {
    WaylandScale.detectAndApply()
    startKoin {
        modules(appModule)
    }
    application {
        Window(onCloseRequest = ::exitApplication, title = "Lampcord") {
            App()
        }
    }
}
