package com.example.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Guild(
    val id: String,
    val name: String? = null,
    val icon: String? = null,
    val banner: String? = null,
    val owner: Boolean? = null,
    val owner_id: String? = null,
    val permissions: String? = null,
    val features: List<String>? = null,
    val roles: List<Role> = emptyList(),
    val emojis: List<Emoji> = emptyList(),
    val stickers: List<Sticker> = emptyList(),
    val members: List<Member>? = null,
    val channels: List<Channel>? = null
)

@Serializable
data class Role(
    val id: String,
    val name: String,
    val color: Int,
    val hoist: Boolean,
    val position: Int,
    val permissions: String,
    val managed: Boolean,
    val mentionable: Boolean,
    val icon: String? = null,
    val unicode_emoji: String? = null
)

@Serializable
data class Member(
    val user: User? = null,
    val nick: String? = null,
    val avatar: String? = null,
    val roles: List<String> = emptyList(),
    val joined_at: String = "",
    val premium_since: String? = null,
    val deaf: Boolean = false,
    val mute: Boolean = false,
    val flags: Int = 0,
    val pending: Boolean? = null,
    val permissions: String? = null,
    val communication_disabled_until: String? = null,
    val presence: PresenceUpdate? = null
)

@Serializable
data class GuildFolder(
    val id: Long? = null,
    val guild_ids: List<String>,
    val name: String? = null,
    val color: Int? = null
)

@Serializable
data class GuildSubscriptionsUpdate(
    val subscriptions: Map<String, GuildSubscription>
)

@Serializable
data class GuildSubscription(
    val typing: Boolean = true,
    val threads: Boolean = true,
    val activities: Boolean = true,
    val members: List<String> = emptyList(),
    val channels: Map<String, List<List<Int>>> = emptyMap()
)

@Serializable
data class MemberListUpdate(
    val guild_id: String,
    val id: String,
    val ops: List<MemberListOp>,
    val member_count: Int? = null,
    val online_count: Int? = null,
    val groups: List<MemberListGroup>? = null
)

@Serializable
data class MemberListOp(
    val op: String,
    val range: List<Int>? = null,
    val items: List<MemberListListItem>? = null,
    val index: Int? = null,
    val item: MemberListListItem? = null
)

@Serializable
data class MemberListListItem(
    val member: Member? = null,
    val group: MemberListGroup? = null
)

@Serializable
data class MemberListGroup(
    val id: String,
    val count: Int? = null,
    val member_count: Int? = null
)
