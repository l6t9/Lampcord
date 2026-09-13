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
) {
    @Serializable
    data class Partial(
        val username: String? = null,
        val avatar: String? = null,
        val banner: String? = null,
        val accent_color: Int? = null,
        val global_name: String? = null,
        val pronouns: String? = null,
        val bio: String? = null
    )
}

@Serializable
data class UserProfile(
    val user: User,
    val user_profile: UserProfileMetadata? = null,
    val guild_id: String? = null,
    val guild_member: Member? = null,
    val guild_member_profile: UserProfileMetadata? = null,
    val presence: PresenceUpdate? = null,
    val activities: List<Activity> = emptyList(),
    val client_status: ClientStatus? = null,
    val badges: List<ProfileBadge> = emptyList(),
    val guild_badges: List<ProfileBadge> = emptyList(),
    val mutual_guilds: List<MutualGuild>? = null,
    val mutual_friends: List<User>? = null,
    @kotlinx.serialization.SerialName("mutual_friends_count")
    val mutual_friends_count: Int? = null,
    val connected_accounts: List<ConnectedAccount> = emptyList(),
    val premium_since: String? = null,
    val premium_guild_since: String? = null
)

@Serializable
data class MutualFriendResponse(
    val id: String,
    val type: Int,
    val user: User
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
data class ProfileEffect(
    val sku_id: String? = null,
    val expires_at: Long? = null
)

@Serializable
data class UserProfileMetadata(
    val bio: String? = null,
    val accent_color: Int? = null,
    val banner: String? = null,
    val theme_colors: List<Int>? = null,
    val pronouns: String? = null,
    val display_name_styles: DisplayNameStyles? = null,
    val profile_effect: ProfileEffect? = null,
    val profile_frame: ProfileEffect? = null
) {
    @Serializable
    data class Partial(
        val bio: String? = null,
        val accent_color: Int? = null,
        val banner: String? = null,
        val theme_colors: List<Int>? = null,
        val pronouns: String? = null,
        val display_name_styles: DisplayNameStyles? = null,
        val profile_effect: ProfileEffect? = null
    )
}

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
    val blocked_message_bar: Boolean? = null,
    val message_display_compact: Boolean? = null
) {
    fun merge(partial: Partial): UserSettings {
        return copy(
            theme = partial.theme ?: theme,
            developer_mode = partial.developer_mode ?: developer_mode,
            render_embeds = partial.render_embeds ?: render_embeds,
            inline_embed_media = partial.inline_embed_media ?: inline_embed_media,
            inline_attachment_media = partial.inline_attachment_media ?: inline_attachment_media,
            blocked_message_bar = partial.blocked_message_bar ?: blocked_message_bar,
            message_display_compact = partial.message_display_compact ?: message_display_compact,
            locale = partial.locale ?: locale,
            restricted_guilds = partial.restricted_guilds ?: restricted_guilds,
            status = partial.status ?: status,
            show_current_game = partial.show_current_game ?: show_current_game,
            guild_folders = partial.guild_folders ?: guild_folders,
            default_guilds_restricted = partial.default_guilds_restricted ?: default_guilds_restricted,
            friend_source_flags = partial.friend_source_flags ?: friend_source_flags,
            explicit_content_filter = partial.explicit_content_filter ?: explicit_content_filter,
            animate_emoji = partial.animate_emoji ?: animate_emoji,
            allow_accessibility_detection = partial.allow_accessibility_detection ?: allow_accessibility_detection,
            animate_stickers = partial.animate_stickers ?: animate_stickers,
            contact_sync_enabled = partial.contact_sync_enabled ?: contact_sync_enabled,
            friend_discovery_flags = partial.friend_discovery_flags ?: friend_discovery_flags,
            custom_status = partial.custom_status ?: custom_status
        )
    }

    @Serializable
    data class Partial(
        val theme: String? = null,
        val developer_mode: Boolean? = null,
        val render_embeds: Boolean? = null,
        val inline_embed_media: Boolean? = null,
        val inline_attachment_media: Boolean? = null,
        val blocked_message_bar: Boolean? = null,
        val message_display_compact: Boolean? = null,
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

@Serializable
data class UserAffinity(
    val user_id: String,
    val affinity: Float
)

@Serializable
data class UserAffinities(
    val user_affinities: List<UserAffinity>,
    val inverse_user_affinities: List<UserAffinity>
)

@Serializable
data class RecentAvatar(
    val id: String,
    val storage_hash: String
)

@Serializable
data class RecentAvatarsResponse(
    val avatars: List<RecentAvatar>
)
