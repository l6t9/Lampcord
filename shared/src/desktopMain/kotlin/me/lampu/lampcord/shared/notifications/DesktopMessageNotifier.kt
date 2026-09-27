package me.lampu.lampcord.shared.notifications

import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.NotificationStore
import java.awt.SystemTray
import java.awt.TrayIcon
import java.awt.image.BufferedImage

class DesktopMessageNotifier(
    private val notificationStore: NotificationStore
) : MessageNotifier {
    override val isInForeground: Boolean = true

    private var trayIcon: TrayIcon? = null

    init {
        try {
            if (SystemTray.isSupported()) {
                val tray = SystemTray.getSystemTray()
                val image = BufferedImage(16, 16, BufferedImage.TYPE_INT_ARGB)
                val g = image.createGraphics()
                g.color = java.awt.Color(114, 137, 218)
                g.fillOval(0, 0, 16, 16)
                g.dispose()
                val icon = TrayIcon(image, "Lampcord")
                icon.isImageAutoSize = true
                tray.add(icon)
                trayIcon = icon
            }
        } catch (_: Exception) {}
    }

    override fun showMessageNotification(data: IncomingNotificationData) {
        if (!Settings.shared.notificationsEnabled) return

        if (Settings.shared.showInAppNotifications) {
            notificationStore.show(data)
        }

        val title = if (data.isDm) data.authorDisplayName else "${data.authorDisplayName} (${data.channelLabel ?: "channel"})"
        val text = previewText(data)

        try {
            trayIcon?.displayMessage(title, text, TrayIcon.MessageType.INFO)
        } catch (_: Exception) {}
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
