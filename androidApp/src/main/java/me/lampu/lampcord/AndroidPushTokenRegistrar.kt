package me.lampu.lampcord

import com.google.firebase.messaging.FirebaseMessaging
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.notifications.PushTokenRegistrar
import me.lampu.lampcord.shared.utils.Logging

@Suppress("DEPRECATION")
class AndroidPushTokenRegistrar(
    private val userApi: UserApi
) : PushTokenRegistrar {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun register(token: String?) {
        if (token != null) {
            upload(token)
            return
        }
        FirebaseMessaging.getInstance().token
            .addOnSuccessListener(::upload)
            .addOnFailureListener { Logging.e("FCM", "Unable to get push token: ${it.message}") }
    }

    private fun upload(token: String) {
        if (token.isBlank()) return
        scope.launch { userApi.registerPushToken(token) }
    }
}
