package me.lampu.lampcord.shared.notifications

import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.NotificationStore
import me.lampu.lampcord.shared.utils.getPlatformName
import javax.sound.sampled.AudioSystem

class DesktopMessageNotifier(
    private val notificationStore: NotificationStore
) : MessageNotifier {
    override val isInForeground: Boolean = true

    override fun showMessageNotification(data: IncomingNotificationData) {
        if (!Settings.shared.notificationsEnabled) return

        if (Settings.shared.notificationSound) {
            playNotificationSound()
        }

        if (Settings.shared.showInAppNotifications) {
            notificationStore.show(data)
        }

        val title = if (data.isDm) data.authorDisplayName else "${data.authorDisplayName} (${data.channelLabel ?: "channel"})"
        val text = previewText(data)

        sendSystemNotification(title, text)
    }

    private fun playNotificationSound() {
        try {
            val stream = javaClass.classLoader.getResourceAsStream("sounds/notification.wav") ?: return
            val audioStream = AudioSystem.getAudioInputStream(stream)
            val clip = AudioSystem.getClip()
            clip.open(audioStream)
            clip.start()
        } catch (_: Exception) {}
    }

    private fun sendSystemNotification(title: String, text: String) {
        val os = getPlatformName()
        if (os == "linux") {
            try {
                ProcessBuilder("notify-send", "-a", "Lampcord", title, text).start()
            } catch (_: Exception) {}
        } else if (os == "macos") {
            try {
                val script = "display notification \"${text.replace("\"", "\\\"")}\" with title \"${title.replace("\"", "\\\"")}\""
                ProcessBuilder("osascript", "-e", script).start()
            } catch (_: Exception) {}
        }
    }

    override fun dismissChannelNotifications(channelId: String) {
        notificationStore.dismissChannel(channelId)
    }

    private fun previewText(data: IncomingNotificationData): String {
        if (!Settings.shared.showMessagePreview) return "New message"
        val message = data.message
        return when {
            message.content.isNotBlank() -> message.content
            message.attachments.isNotEmpty() -> "Sent an attachment"
            message.sticker_items?.isNotEmpty() == true -> "Sent a sticker"
            message.embeds.isNotEmpty() -> message.embeds.first().title ?: "Sent an embed"
            else -> "New message"
        }
    }
}
