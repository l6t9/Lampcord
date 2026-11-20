package me.lampu.lampcord

import com.google.firebase.messaging.FirebaseMessaging
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.notifications.PushTokenRegistrar
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Logging

@Suppress("DEPRECATION")
class AndroidPushTokenRegistrar(
    private val userApi: UserApi
) : PushTokenRegistrar {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var lastToken: String? = null

    override fun register(token: String?) {
        if (token != null) {
            upload(token)
            return
        }
        FirebaseMessaging.getInstance().token
            .addOnSuccessListener(::upload)
            .addOnFailureListener { Logging.e("FCM", "Unable to get push token: ${it.message}") }
    }

    override fun unregister() {
        val authToken = Settings.shared.discordToken
        if (authToken.isBlank()) return
        val fcmToken = lastToken
        if (fcmToken != null) {
            scope.launch { userApi.unregisterPushToken(fcmToken, authToken) }
            return
        }
        FirebaseMessaging.getInstance().token
            .addOnSuccessListener { token -> scope.launch { userApi.unregisterPushToken(token, authToken) } }
            .addOnFailureListener { Logging.e("FCM", "Unable to get push token for unregistration: ${it.message}") }
    }

    private fun upload(token: String) {
        if (token.isBlank()) return
        lastToken = token
        scope.launch { userApi.registerPushToken(token) }
    }
}
