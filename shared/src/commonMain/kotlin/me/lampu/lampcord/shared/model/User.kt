package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class User(
    val id: String,
    val username: String = "",
    val discriminator: String = "0000",
    val avatar: String? = null,
    val global_name: String? = null,
    val accent_color: Int? = null,
    val banner: String? = null,
    val pronouns: String? = null,
    val bio: String? = null,
    val public_flags: Int? = null,
    val premium_type: Int? = null
)

@Serializable
data class UserProfile(
    val user: User,
    val user_profile: UserProfileMetadata? = null,
    val guild_member: Member? = null,
    val guild_member_profile: UserProfileMetadata? = null,
    val badges: List<ProfileBadge> = emptyList(),
    val guild_badges: List<ProfileBadge> = emptyList(),
    val connected_accounts: List<ConnectedAccount> = emptyList()
)

@Serializable
data class ConnectedAccount(
    val id: String,
    val name: String,
    val type: String,
    val verified: Boolean = false
)

@Serializable
data class ProfileBadge(
    val id: String,
    val icon: String,
    val description: String
)

@Serializable
data class UserProfileMetadata(
    val bio: String? = null,
    val accent_color: Int? = null,
    val banner: String? = null,
    val theme_colors: List<Int>? = null,
    val pronouns: String? = null
)

@Serializable
data class UserSettings(
    val guild_positions: List<String> = emptyList(),
    val guild_folders: List<GuildFolder> = emptyList(),
    val friend_source_flags: FriendSourceFlags? = null,
    val theme: String? = null,
    val developer_mode: Boolean? = null,
    val locale: String? = null,
    val status: String? = null,
    val custom_status: CustomStatus? = null
)

@Serializable
data class FriendSourceFlags(val all: Boolean? = null)

@Serializable
data class CustomStatus(val text: String? = null, val emoji_id: String? = null, val emoji_name: String? = null)
