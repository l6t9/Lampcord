package me.lampu.lampcord

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.notifications.IncomingNotificationData

internal fun Map<String, String>.toIncomingNotificationData(): IncomingNotificationData? {
    if (this["type"] != "MESSAGE_CREATE") return null

    val messageId = this["message_id"] ?: return null
    val channelId = this["channel_id"] ?: return null
    val userId = this["user_id"] ?: return null
    val guildId = this["guild_id"]?.takeUnless { it.isBlank() || it == "-1" }
    val username = this["user_username"] ?: "Unknown"
    val avatar = this["user_avatar"]
    val channelName = this["channel_name"]
    val channelType = this["channel_type"]?.toIntOrNull()

    return IncomingNotificationData(
        message = Message(
            id = messageId,
            channel_id = channelId,
            author = User(
                id = userId,
                username = username,
                discriminator = this["user_discriminator"],
                avatar = avatar
            ),
            content = this["message_content"].orEmpty(),
            type = this["message_type_"]?.toIntOrNull(),
            guild_id = guildId
        ),
        channel = Channel(
            id = channelId,
            type = channelType,
            guild_id = guildId,
            name = channelName,
            icon = this["channel_icon"]
        ),
        guild = guildId?.let {
            Guild(id = it, name = this["guild_name"], icon = this["guild_icon"])
        },
        authorDisplayName = username,
        authorAvatarUrl = avatar?.let {
            "https://cdn.discordapp.com/avatars/$userId/$it.png?size=128"
        },
        channelLabel = when {
            guildId != null -> channelName?.let { "#$it" }
            channelType == 3 -> channelName
            else -> null
        },
        isDm = guildId == null,
        isMention = false
    )
}
