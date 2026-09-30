package me.lampu.lampcord.shared.notifications

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import kotlin.coroutines.resume
import platform.Foundation.NSError
import platform.UIKit.UIApplication
import platform.UIKit.UIApplicationState
import platform.UserNotifications.UNAuthorizationOptionAlert
import platform.UserNotifications.UNAuthorizationOptionBadge
import platform.UserNotifications.UNAuthorizationOptionSound
import platform.UserNotifications.UNAuthorizationOptions
import platform.UserNotifications.UNMutableNotificationContent
import platform.UserNotifications.UNNotificationAction
import platform.UserNotifications.UNNotificationCategory
import platform.UserNotifications.UNNotificationRequest
import platform.UserNotifications.UNNotificationSound
import platform.UserNotifications.UNUserNotificationCenter
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.NotificationStore
import me.lampu.lampcord.shared.utils.Logging

@OptIn(ExperimentalForeignApi::class)
class IosMessageNotifier(
    private val notificationStore: NotificationStore
) : MessageNotifier {

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    private val deliveredByChannel = mutableMapOf<String, MutableSet<String>>()

    override val isInForeground: Boolean
        get() = UIApplication.sharedApplication.applicationState ==
            UIApplicationState.UIApplicationStateActive

    override fun showMessageNotification(data: IncomingNotificationData) {
        if (data.pushType == NotificationPushType.CALL_RING) return

        if (isInForeground) {
            notificationStore.show(data)
            return
        }
        scope.launch { present(data) }
    }

    override fun dismissChannelNotifications(channelId: String) {
        deliveredByChannel.remove(channelId)
        UNUserNotificationCenter.currentNotificationCenter().removeAllDeliveredNotifications()
        notificationStore.dismissChannel(channelId)
    }

    override fun dismissAllNotifications() {
        deliveredByChannel.clear()
        UNUserNotificationCenter.currentNotificationCenter().removeAllDeliveredNotifications()
        notificationStore.dismissAll()
    }

    private suspend fun present(data: IncomingNotificationData) = withContext(Dispatchers.Main) {
        if (!Settings.shared.notificationsEnabled) return@withContext
        if (!ensurePermission()) return@withContext

        val channelId = data.message.channel_id
        val conversation = if (data.isDm) {
            data.authorDisplayName
        } else {
            data.channelLabel ?: data.authorDisplayName
        }

        val content = UNMutableNotificationContent().apply {
            setTitle(conversation)
            setBody(preview(data))
            setCategoryIdentifier(MESSAGE_CATEGORY)
            setUserInfo(
                mapOf(
                    "channel_id" to channelId,
                    "guild_id" to (data.message.guild_id ?: ""),
                )
            )
            if (Settings.shared.notificationSound) {
                setSound(UNNotificationSound.defaultSound)
            }
        }

        val center = UNUserNotificationCenter.currentNotificationCenter()
        center.setNotificationCategories(setOf(category()))

        val identifier = "$channelId:${data.message.id}"
        center.addNotificationRequest(
            UNNotificationRequest.requestWithIdentifier(identifier, content, null)
        ) { error: NSError? ->
            if (error != null) {
                Logging.w(TAG, "Unable to post a notification for $channelId")
            }
        }
        deliveredByChannel.getOrPut(channelId) { mutableSetOf() }.add(identifier)
    }

    private fun category() = UNNotificationCategory.categoryWithIdentifier(
        identifier = MESSAGE_CATEGORY,
        actions = emptyList<UNNotificationAction>(),
        intentIdentifiers = emptyList<String>(),
        options = 0uL
    )

    private suspend fun ensurePermission(): Boolean =
        suspendCancellableCoroutine { continuation ->
            val options: UNAuthorizationOptions =
                UNAuthorizationOptionAlert or
                    UNAuthorizationOptionSound or
                    UNAuthorizationOptionBadge
            UNUserNotificationCenter.currentNotificationCenter()
                .requestAuthorizationWithOptions(options) { granted, error ->
                    if (error != null) {
                        Logging.w(TAG, "Notification permission request failed")
                    }
                    if (continuation.isActive) continuation.resume(granted)
                }
        }

    private fun preview(data: IncomingNotificationData): String {
        if (!Settings.shared.showMessagePreview) return "New message"
        val message = data.message
        return when {
            message.content.isNotBlank() -> message.content
            message.attachments.isNotEmpty() -> "Sent an attachment"
            message.sticker_items?.isNotEmpty() == true -> "Sent a sticker"
            message.embeds.isNotEmpty() -> "Sent an embed"
            else -> "New message"
        }
    }

    private companion object {
        const val TAG = "notifications"
        const val MESSAGE_CATEGORY = "me.lampu.lampcord.MESSAGES"
    }
}