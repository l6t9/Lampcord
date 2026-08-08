package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class AvatarDecorationData(
    val asset: String,
    val sku_id: String? = null
)

@Serializable
data class Nameplate(
    val asset: String,
    val palette: String? = null
)

@Serializable
data class Collectibles(
    val nameplate: Nameplate? = null,
    val avatar_decoration: AvatarDecorationData? = null
)

@Serializable
data class PrimaryGuild(
    val guild_id: String? = null,
    val identity_guild_id: String? = null,
    val identity_enabled: Boolean? = null,
    val tag: String? = null,
    val badge: String? = null
)

@Serializable
data class DisplayNameStyles(
    val font_id: Int? = null,
    val effect_id: Int? = null,
    val colors: List<Int>? = null
)

@Serializable
data class User(
    val id: String,
    val username: String? = null,
    val discriminator: String? = null,
    val avatar: String? = null,
    val bot: Boolean? = null,
    val system: Boolean? = null,
    val mfa_enabled: Boolean? = null,
    val banner: String? = null,
    val accent_color: Int? = null,
    val locale: String? = null,
    val verified: Boolean? = null,
    val email: String? = null,
    val flags: Int? = null,
    val premium_type: Int? = null,
    val public_flags: Int? = null,
    val global_name: String? = null,
    val pronouns: String? = null,
    val bio: String? = null,
    val avatar_decoration_data: AvatarDecorationData? = null,
    val collectibles: Collectibles? = null,
    val primary_guild: PrimaryGuild? = null,
    val display_name_styles: DisplayNameStyles? = null
)

@Serializable
data class UserProfile(
    val user: User,
    val user_profile: UserProfileMetadata? = null,
    val guild_member: Member? = null,
    val guild_member_profile: UserProfileMetadata? = null,
    val badges: List<ProfileBadge> = emptyList(),
    val guild_badges: List<ProfileBadge> = emptyList(),
    val mutual_guilds: List<MutualGuild>? = null,
    val connected_accounts: List<ConnectedAccount> = emptyList(),
    val premium_since: String? = null,
    val premium_guild_since: String? = null
)

@Serializable
data class MutualGuild(
    val id: String,
    val nick: String? = null
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
    val pronouns: String? = null,
    val display_name_styles: DisplayNameStyles? = null
)

@Serializable
data class UserSettings(
    val guild_positions: List<JsonElement> = emptyList(),
    val guild_folders: List<GuildFolder> = emptyList(),
    val friend_source_flags: FriendSourceFlags? = null,
    val theme: String? = null,
    val developer_mode: Boolean? = null,
    val locale: String? = null,
    val status: String? = null,
    val custom_status: CustomStatus? = null,
    val inline_attachment_media: Boolean? = null,
    val inline_embed_media: Boolean? = null,
    val render_embeds: Boolean? = null,
    val animate_emoji: Boolean? = null,
    val animate_stickers: Int? = null,
    val explicit_content_filter: Int? = null,
    val allow_accessibility_detection: Boolean? = null,
    val contact_sync_enabled: Boolean? = null,
    val default_guilds_restricted: Boolean? = null,
    val friend_discovery_flags: Int? = null,
    val restricted_guilds: List<String> = emptyList(),
    val show_current_game: Boolean? = null,
    val blocked_message_bar: Boolean? = null
) {
    @Serializable
    data class Partial(
        val theme: String? = null,
        val developer_mode: Boolean? = null,
        val render_embeds: Boolean? = null,
        val inline_embed_media: Boolean? = null,
        val inline_attachment_media: Boolean? = null,
        val blocked_message_bar: Boolean? = null,
        val locale: String? = null,
        val restricted_guilds: List<String>? = null,
        val status: String? = null,
        val show_current_game: Boolean? = null,
        val guild_folders: List<GuildFolder>? = null,
        val guild_positions: List<String>? = null,
        val default_guilds_restricted: Boolean? = null,
        val friend_source_flags: FriendSourceFlags? = null,
        val explicit_content_filter: Int? = null,
        val animate_emoji: Boolean? = null,
        val allow_accessibility_detection: Boolean? = null,
        val animate_stickers: Int? = null,
        val contact_sync_enabled: Boolean? = null,
        val friend_discovery_flags: Int? = null,
        val custom_status: CustomStatus? = null
    )
}

@Serializable
data class FriendSourceFlags(
    val all: Boolean? = null,
    val mutual_friends: Boolean? = null,
    val mutual_guilds: Boolean? = null
)

@Serializable
data class CustomStatus(val text: String? = null, val emoji_id: String? = null, val emoji_name: String? = null)

@Serializable
data class UserNoteUpdate(
    val id: String,
    val note: String
)
