package me.lampu.lampcord

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.notifications.IncomingNotificationData
import me.lampu.lampcord.shared.notifications.NotificationPushType

private object PushKeys {
    const val TYPE = "type"
    const val MESSAGE_ID = "message_id"
    const val MESSAGE_TYPE = "message_type_"
    const val MESSAGE_CONTENT = "message_content"
    const val CHANNEL_ID = "channel_id"
    const val CHANNEL_TYPE = "channel_type"
    const val CHANNEL_NAME = "channel_name"
    const val CHANNEL_ICON = "channel_icon"
    const val CHANNEL_IDS = "channel_ids"
    const val GUILD_ID = "guild_id"
    const val GUILD_NAME = "guild_name"
    const val GUILD_ICON = "guild_icon"
    const val USER_ID = "user_id"
    const val USER_USERNAME = "user_username"
    const val USER_DISCRIMINATOR = "user_discriminator"
    const val USER_AVATAR = "user_avatar"
    const val TITLE = "title"
}

private const val UNSET = "-1"

private const val GROUP_DM = 3

private fun Map<String, String>.longOrNull(key: String): String? =
    this[key]?.takeUnless { it.isBlank() || it == UNSET }

internal fun Map<String, String>.ackChannelIds(): List<String> =
    this[PushKeys.CHANNEL_IDS]
        .orEmpty()
        .split(',')
        .mapNotNull { it.trim().toLongOrNull()?.takeIf { id -> id != -1L }?.toString() }

internal fun Map<String, String>.toIncomingNotificationData(): IncomingNotificationData? {
    val pushType = when (this[PushKeys.TYPE]) {
        "MESSAGE_CREATE" -> NotificationPushType.MESSAGE_CREATE
        "CALL_RING" -> NotificationPushType.CALL_RING
        "RELATIONSHIP_ADD" -> NotificationPushType.RELATIONSHIP_ADD
        else -> return null
    }

    val channelId = longOrNull(PushKeys.CHANNEL_ID) ?: return null
    val messageId = this[PushKeys.MESSAGE_ID]?.takeUnless { it.isBlank() } ?: return null
    val userId = longOrNull(PushKeys.USER_ID) ?: UNSET
    val guildId = longOrNull(PushKeys.GUILD_ID)
    val username = this[PushKeys.USER_USERNAME] ?: this[PushKeys.TITLE] ?: "Unknown"
    val avatar = this[PushKeys.USER_AVATAR]?.takeUnless { it.isBlank() }
    val channelName = this[PushKeys.CHANNEL_NAME]?.takeUnless { it.isBlank() }
    val channelType = this[PushKeys.CHANNEL_TYPE]?.toIntOrNull() ?: -1

    return IncomingNotificationData(
        message = Message(
            id = messageId,
            channel_id = channelId,
            author = User(
                id = userId,
                username = username,
                discriminator = this[PushKeys.USER_DISCRIMINATOR],
                avatar = avatar
            ),
            content = this[PushKeys.MESSAGE_CONTENT].orEmpty(),
            type = this[PushKeys.MESSAGE_TYPE]?.toIntOrNull() ?: 0,
            guild_id = guildId
        ),
        channel = Channel(
            id = channelId,
            type = channelType,
            guild_id = guildId,
            name = channelName,
            icon = this[PushKeys.CHANNEL_ICON]
        ),
        guild = guildId?.let {
            Guild(id = it, name = this[PushKeys.GUILD_NAME], icon = this[PushKeys.GUILD_ICON])
        },
        authorDisplayName = username,
        authorAvatarUrl = avatar?.let {
            "https://cdn.discordapp.com/avatars/$userId/$it.png?size=128"
        },
        channelLabel = when {
            guildId != null -> channelName?.let { "#$it" }
            channelType == GROUP_DM -> channelName
            else -> null
        },
        isDm = guildId == null,
        // A push only exists because the server decided this mention is worth sending.
        isMention = true,
        pushType = pushType
    )
}
