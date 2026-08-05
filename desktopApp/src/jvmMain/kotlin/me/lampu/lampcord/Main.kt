package me.lampu.lampcord

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.ui.App
import me.lampu.lampcord.ui.WaylandDensityProvider
import me.lampu.lampcord.utils.WaylandScale
import org.koin.core.context.startKoin

fun main() {
    WaylandScale.detectAndApply()
    
    startKoin {
        modules(appModule)
    }

    application {
        Window(
            onCloseRequest = ::exitApplication, 
            title = "Lampcord"
        ) {
            WaylandDensityProvider {
                App()
            }
        }
    }
}
