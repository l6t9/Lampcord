package me.lampu.lampcord

import android.os.Build
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.settings.Settings
import java.net.HttpURLConnection
import java.net.URL

object PushRelayManager {
    private val scope = CoroutineScope(Dispatchers.IO)

    fun registerCurrentDevice() {
        val relayUrl = Settings.shared.pushRelayServerUrl.trim().removeSuffix("/")
        val token = Settings.shared.discordToken
        val fcmToken = Settings.shared.fcmToken

        if (relayUrl.isBlank() || token.isBlank() || fcmToken.isBlank()) return

        scope.launch {
            try {
                val url = URL("$relayUrl/api/register")
                val conn = url.openConnection() as HttpURLConnection
                conn.requestMethod = "POST"
                conn.setRequestProperty("Content-Type", "application/json")
                conn.doOutput = true

                val jsonPayload = """
                    {
                        "discord_token": "$token",
                        "fcm_token": "$fcmToken",
                        "device_id": "${Build.MODEL}"
                    }
                """.trimIndent()

                conn.outputStream.use { os ->
                    os.write(jsonPayload.toByteArray(Charsets.UTF_8))
                }

                val responseCode = conn.responseCode
                println("PushRelayManager: Registration response code = $responseCode")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun unregisterDevice() {
        val relayUrl = Settings.shared.pushRelayServerUrl.trim().removeSuffix("/")
        val token = Settings.shared.discordToken
        val fcmToken = Settings.shared.fcmToken

        if (relayUrl.isBlank() || token.isBlank() || fcmToken.isBlank()) return

        scope.launch {
            try {
                val url = URL("$relayUrl/api/unregister")
                val conn = url.openConnection() as HttpURLConnection
                conn.requestMethod = "POST"
                conn.setRequestProperty("Content-Type", "application/json")
                conn.doOutput = true

                val jsonPayload = """
                    {
                        "discord_token": "$token",
                        "fcm_token": "$fcmToken"
                    }
                """.trimIndent()

                conn.outputStream.use { os ->
                    os.write(jsonPayload.toByteArray(Charsets.UTF_8))
                }

                println("PushRelayManager: Unregistered device from push relay")
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}
