package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class Channel(
    val id: String,
    val type: Int,
    val guild_id: String? = null,
    val position: Int? = null,
    val name: String? = null,
    val topic: String? = null,
    val nsfw: Boolean? = null,
    var last_message_id: JsonElement? = null,
    val parent_id: String? = null,
    val recipients: List<User>? = null,
    val icon: String? = null,
    val thread_metadata: ThreadMetadata? = null,
    val message_count: Int? = null,
    val member_count: Int? = null,
    val total_message_sent: Int? = null,
    val available_tags: List<ForumTag>? = null,
    val applied_tags: List<String>? = null,
    val permission_overwrites: List<PermissionOverwrite>? = null
)

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
    val type: Int = 0,
    val allow: String = "0",
    val deny: String = "0"
)

@Serializable
data class ThreadMetadata(
    val archived: Boolean,
    val auto_archive_duration: Int,
    val archive_timestamp: String,
    val locked: Boolean,
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
data class ThreadListSync(
    val guild_id: String,
    val channel_ids: List<String>? = null,
    val threads: List<Channel> = emptyList(),
    val members: List<ThreadMember> = emptyList()
)

@Serializable
data class ThreadDeleteEvent(
    val id: String,
    val guild_id: String? = null,
    val parent_id: String? = null,
    val type: Int? = null
)

@Serializable
data class ThreadMember(
    val id: String? = null,
    val user_id: String? = null,
    val join_timestamp: String? = null,
    val flags: Int? = null
)
