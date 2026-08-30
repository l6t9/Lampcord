package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*

@Serializable
data class Channel(
    val id: String,
    val type: Int? = null,
    val guild_id: String? = null,
    val owner_id: String? = null,
    val position: Int? = null,
    val name: String? = null,
    val topic: String? = null,
    val nsfw: Boolean? = null,
    var last_message_id: JsonElement? = null,
    val parent_id: String? = null,
    val recipients: List<User>? = null,
    val recipient_ids: List<String>? = null,
    val icon: String? = null,
    val thread_metadata: ThreadMetadata? = null,
    val message_count: Int? = null,
    val member_count: Int? = null,
    val total_message_sent: Int? = null,
    val available_tags: List<ForumTag>? = null,
    val applied_tags: List<String>? = null,
    val reactions: List<MessageReaction>? = null,
    val message: Message? = null,
    val permission_overwrites: List<PermissionOverwrite>? = null,
    val member_list_id: String? = null,
    val flags: Int? = null
) {
    fun lastMessageId(): String? = last_message_id?.jsonPrimitive?.contentOrNull
}

@Serializable
data class ForumTag(
    val id: String,
    val name: String,
    val moderated: Boolean,
    val emoji_id: String? = null,
    val emoji_name: String? = null
)

@Serializable
data class PermissionOverwrite(
    val id: String,
    val type: JsonElement,
    val allow: JsonElement? = null,
    val deny: JsonElement? = null,
    val allow_new: String? = null,
    val deny_new: String? = null
) {

    fun allowString(): String {
        return allow_new ?: if (allow is JsonPrimitive) allow.content else "0"
    }

    fun denyString(): String {
        return deny_new ?: if (deny is JsonPrimitive) deny.content else "0"
    }
}

@Serializable
data class ThreadMetadata(
    val archived: Boolean? = null,
    val auto_archive_duration: Int? = null,
    val archive_timestamp: String? = null,
    val locked: Boolean? = null,
    val invitable: Boolean? = null,
    val create_timestamp: String? = null
)

@Serializable
data class ThreadListResponse(
    val threads: List<Channel> = emptyList(),
    val members: List<ThreadMember> = emptyList(),
    val has_more: Boolean? = null
)

@Serializable
data class ThreadMember(
    val id: String? = null,
    val user_id: String? = null,
    val join_timestamp: String? = null,
    val flags: Int? = null
)
