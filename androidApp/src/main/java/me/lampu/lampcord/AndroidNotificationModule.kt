package me.lampu.lampcord

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import me.lampu.lampcord.shared.notifications.MessageNotifier
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val androidNotificationModule = module {
    single<MessageNotifier> {
        AndroidMessageNotifier(
            context = androidContext(),
            notificationStore = get(),
            userStore = get(),
            scope = CoroutineScope(Dispatchers.Main)
        )
    }
}
