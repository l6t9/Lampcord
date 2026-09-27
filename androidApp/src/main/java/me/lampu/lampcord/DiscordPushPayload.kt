package me.lampu.lampcord

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.notifications.IncomingNotificationData

internal fun Map<String, String>.toIncomingNotificationData(): IncomingNotificationData? {
    val payloadType = this["type"] ?: "MESSAGE_CREATE"
    if (payloadType != "MESSAGE_CREATE" && payloadType != "CALL_CREATE" && payloadType != "RELATIONSHIP_ADD") return null

    val messageId = this["message_id"] ?: this["id"] ?: System.currentTimeMillis().toString()
    val channelId = this["channel_id"] ?: return null
    val userId = this["user_id"] ?: this["author_id"] ?: "0"
    val guildId = this["guild_id"]?.takeUnless { it.isBlank() || it == "-1" }
    val username = this["user_username"] ?: this["author_name"] ?: this["title"] ?: "Unknown"
    val avatar = this["user_avatar"] ?: this["avatar_hash"]
    val channelName = this["channel_name"]
    val channelType = this["channel_type"]?.toIntOrNull()
    val content = this["message_content"] ?: this["content"] ?: this["body"] ?: this["text"] ?: ""

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
            content = content,
            type = this["message_type_"]?.toIntOrNull() ?: 0,
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
        isMention = this["mention"] == "true" || this["is_mention"] == "true"
    )
}
