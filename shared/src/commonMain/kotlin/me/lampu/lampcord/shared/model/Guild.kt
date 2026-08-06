package me.lampu.lampcord.shared.model

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
    val channels: List<Channel>? = null,
    val welcome_screen: WelcomeScreen? = null
)

@Serializable
data class WelcomeScreen(
    val description: String? = null,
    val welcome_channels: List<WelcomeScreenChannel> = emptyList()
)

@Serializable
data class WelcomeScreenChannel(
    val channel_id: String,
    val description: String,
    val emoji_id: String? = null,
    val emoji_name: String? = null
)

@Serializable
data class Onboarding(
    val guild_id: String,
    val prompts: List<OnboardingPrompt> = emptyList(),
    val default_channel_ids: List<String> = emptyList(),
    val enabled: Boolean = false,
    val mode: Int = 0
)

@Serializable
data class OnboardingPrompt(
    val id: String,
    val type: Int,
    val options: List<OnboardingPromptOption> = emptyList(),
    val title: String,
    val single_select: Boolean = false,
    val required: Boolean = false,
    val in_onboarding: Boolean = false
)

@Serializable
data class OnboardingPromptOption(
    val id: String,
    val channel_ids: List<String> = emptyList(),
    val role_ids: List<String> = emptyList(),
    val emoji: Emoji? = null,
    val title: String,
    val description: String? = null
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
    val presence: PresenceUpdate? = null,
    val avatar_decoration_data: AvatarDecorationData? = null,
    val collectibles: Collectibles? = null,
    val display_name_styles: DisplayNameStyles? = null
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

@Serializable
data class UserGuildSettings(
    val guild_id: String? = null,
    val muted: Boolean = false,
    val hide_muted_channels: Boolean = false,
    val suppress_everyone: Boolean = false,
    val suppress_roles: Boolean = false,
    val message_notifications: Int = 0,
    val mobile_push: Boolean = true,
    val mute_scheduled_events: Boolean = false,
    val channel_overrides: List<ChannelOverride> = emptyList(),
    val flags: Int = 0,
    val version: Int = 0
) {
    @Serializable
    data class Partial(
        val muted: Boolean? = null,
        val channel_overrides: List<ChannelOverride>? = null
    )
}

@Serializable
data class ChannelOverride(
    val channel_id: String,
    val muted: Boolean = false,
    val message_notifications: Int = 3, // inherit
    val collapsed: Boolean = false,
    val flags: Int = 0
)
