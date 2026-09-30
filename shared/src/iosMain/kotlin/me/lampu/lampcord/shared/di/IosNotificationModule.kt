package me.lampu.lampcord.shared.di

import kotlinx.cinterop.ExperimentalForeignApi
import me.lampu.lampcord.shared.notifications.IosMessageNotifier
import me.lampu.lampcord.shared.notifications.MessageNotifier
import org.koin.dsl.module

@OptIn(ExperimentalForeignApi::class)
val iosNotificationModule = module {
    single<MessageNotifier> { IosMessageNotifier(get()) }
}