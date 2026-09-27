package me.lampu.lampcord.shared.di

import me.lampu.lampcord.shared.notifications.DesktopMessageNotifier
import me.lampu.lampcord.shared.notifications.MessageNotifier
import org.koin.dsl.module

val desktopNotificationModule = module {
    single<MessageNotifier> { DesktopMessageNotifier(get()) }
}
