package me.lampu.lampcord.shared

import kotlin.experimental.ExperimentalObjCName
import kotlin.experimental.ExperimentalObjCRefinement
import kotlin.native.ObjCName
import me.lampu.lampcord.shared.di.appModule
import me.lampu.lampcord.shared.di.iosNotificationModule
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.createSettings
import me.lampu.lampcord.shared.utils.Logging
import org.koin.core.context.startKoin

@OptIn(ExperimentalObjCRefinement::class, ExperimentalObjCName::class)
@ObjCName("LampcordPlatform")
object LampcordPlatform {
    private var started = false

    fun initialize() {
        if (started) return
        started = true

        createSettings()
        Logging.debugEnabled = Settings.shared.verboseLogging

        startKoin {
            modules(appModule, iosNotificationModule)
        }
    }
}
