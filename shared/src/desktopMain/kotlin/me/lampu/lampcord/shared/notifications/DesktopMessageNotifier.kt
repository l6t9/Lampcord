package me.lampu.lampcord.shared.notifications

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.NotificationStore
import me.lampu.lampcord.shared.utils.getPlatformName
import java.io.File
import java.net.URI
import java.nio.file.Files
import java.util.concurrent.ConcurrentHashMap
import javax.sound.sampled.AudioSystem

class DesktopMessageNotifier(
    private val notificationStore: NotificationStore
) : MessageNotifier {
    override val isInForeground: Boolean = true

    // The gateway collector calls showMessageNotification on Dispatchers.Main, so fetching the
    // avatar has to happen off-thread or the window freezes while it downloads.
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val iconDir = notificationIconDir()
    private val iconCache = ConcurrentHashMap<String, File>()

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
        val avatarUrl = data.authorAvatarUrl

        // Only notify-send accepts a per-notification image, so skip the download everywhere else.
        if (avatarUrl.isNullOrBlank() || iconDir == null || getPlatformName() != "linux") {
            sendSystemNotification(title, text, null)
        } else {
            scope.launch { sendSystemNotification(title, text, avatarIcon(avatarUrl)) }
        }
    }

    // Belongs in the app's own cache, not shared /tmp, so it is owned by Lampcord and goes away
    // with the rest of the cache. Falls back to a temp dir only if neither is writable.
    private fun notificationIconDir(): File? {
        val cacheRoot: String? = System.getenv("XDG_CACHE_HOME")
            ?: System.getProperty("user.home")?.let { "$it/.cache" }
        return runCatching<File> {
            val dir = cacheRoot
                ?.let { File(it, "lampcord/notification-icons") }
                ?.takeIf { it.isDirectory || it.mkdirs() }
                ?: Files.createTempDirectory("lampcord-notif").toFile()
            // A stale icon shows the wrong face next to the right name, so never reuse one.
            dir.listFiles()?.forEach { it.delete() }
            dir
        }.getOrNull()
    }

    // notify-send takes a file path and keys its icon cache on it, so one stable path per
    // avatar is both correct and avoids re-reading the image on every message. Capped by count
    // because nothing else bounds it: the cache would otherwise keep one file per distinct
    // sender for the lifetime of the profile directory.
    private fun avatarIcon(url: String): File? {
        iconCache[url]?.let { if (it.isFile) return it }
        val dir = iconDir ?: return null
        return try {
            val target = File(dir, Integer.toHexString(url.hashCode()) + ".img")
            URI(url).toURL().openStream().use { input -> target.outputStream().use { input.copyTo(it) } }
            iconCache[url] = target
            pruneIcons(dir)
            target
        } catch (_: Exception) {
            null
        }
    }

    private fun pruneIcons(dir: File) {
        val files = dir.listFiles()?.filter { it.isFile } ?: return
        if (files.size <= MAX_CACHED_ICONS) return
        files.sortedBy { it.lastModified() }
            .take(files.size - MAX_CACHED_ICONS)
            .forEach {
                iconCache.remove(it.name, it)
                it.delete()
            }
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

    private fun sendSystemNotification(title: String, text: String, icon: File?) {
        val os = getPlatformName()
        if (os == "linux") {
            try {
                val cmd = mutableListOf("notify-send", "-a", "Lampcord")
                if (icon != null && icon.isFile) {
                    cmd += listOf("-i", icon.absolutePath)
                }
                cmd += listOf(title, text)
                ProcessBuilder(cmd).start()
            } catch (_: Exception) {}
        } else if (os == "macos") {
            try {
                // osascript display notification has no per-notification image, so macOS keeps
                // the app icon regardless of the avatar.
                val script = "display notification \"${text.replace("\"", "\\\"")}\" with title \"${title.replace("\"", "\\\"")}\""
                ProcessBuilder("osascript", "-e", script).start()
            } catch (_: Exception) {}
        }
    }

    override fun dismissChannelNotifications(channelId: String) {
        notificationStore.dismissChannel(channelId)
    }

    private companion object {
        const val MAX_CACHED_ICONS = 64
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
