package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.model.*

class EntityStore(
    private val userStore: UserStore
) {
    private val _guilds = MutableStateFlow<Map<String, Guild>>(emptyMap())
    val guilds: StateFlow<Map<String, Guild>> = _guilds.asStateFlow()

    private val _channels = MutableStateFlow<Map<String, Channel>>(emptyMap())
    val channels: StateFlow<Map<String, Channel>> = _channels.asStateFlow()

    fun updateGuild(guild: Guild) {
        _guilds.update { current ->
            val existing = current[guild.id]
            val updated = existing?.copy(
                name = guild.name ?: existing.name,
                icon = guild.icon ?: existing.icon,
                banner = guild.banner ?: existing.banner,
                splash = guild.splash ?: existing.splash,
                description = guild.description ?: existing.description,
                features = guild.features ?: existing.features,
                roles = guild.roles.ifEmpty { existing.roles },
                emojis = guild.emojis.ifEmpty { existing.emojis },
                stickers = guild.stickers.ifEmpty { existing.stickers },
                afk_channel_id = guild.afk_channel_id ?: existing.afk_channel_id,
                afk_timeout = guild.afk_timeout ?: existing.afk_timeout,
                system_channel_id = guild.system_channel_id ?: existing.system_channel_id,
                system_channel_flags = guild.system_channel_flags ?: existing.system_channel_flags,
                rules_channel_id = guild.rules_channel_id ?: existing.rules_channel_id,
                public_updates_channel_id = guild.public_updates_channel_id ?: existing.public_updates_channel_id,
                preferred_locale = guild.preferred_locale ?: existing.preferred_locale,
                verification_level = guild.verification_level ?: existing.verification_level,
                explicit_content_filter = guild.explicit_content_filter ?: existing.explicit_content_filter,
                default_message_notifications = guild.default_message_notifications ?: existing.default_message_notifications,
                mfa_level = guild.mfa_level ?: existing.mfa_level,
                nsfw_level = guild.nsfw_level ?: existing.nsfw_level,
                unavailable = guild.unavailable ?: existing.unavailable,
                member_count = guild.member_count ?: guild.approximate_member_count ?: existing.member_count,
                approximate_member_count = guild.approximate_member_count ?: existing.approximate_member_count,
                approximate_presence_count = guild.approximate_presence_count ?: existing.approximate_presence_count,
                premium_tier = guild.premium_tier ?: existing.premium_tier,
                premium_subscription_count = guild.premium_subscription_count ?: existing.premium_subscription_count
            )
                ?: guild
            current + (guild.id to updated)
        }
        
        guild.channels?.forEach { channel ->
            updateChannel(channel.copy(guild_id = guild.id))
        }
        
        guild.members?.forEach { member ->
            val userId = member.userId() ?: return@forEach
            userStore.cacheMember(guild.id, userId, member)
        }
    }

    fun removeGuild(guildId: String) {
        _guilds.update { it - guildId }
        _channels.update { current ->
            current.filterValues { it.guild_id != guildId }
        }
    }

    fun updateChannel(channel: Channel) {
        _channels.update { current ->
            val existing = current[channel.id]
            val type = channel.type ?: existing?.type
            val isDm = type == 1 || type == 3
            val newGuildId = if (isDm) null else (channel.guild_id ?: existing?.guild_id)

            val updatedRecipients = if (!channel.recipients.isNullOrEmpty()) channel.recipients else existing?.recipients
            val updatedRecipientIds = if (!channel.recipient_ids.isNullOrEmpty()) {
                channel.recipient_ids
            } else {
                existing?.recipient_ids ?: updatedRecipients?.map { it.id }
            }

            val updated = existing?.copy(
                type = type,
                guild_id = newGuildId,
                position = channel.position ?: existing.position,
                name = channel.name ?: existing.name,
                topic = channel.topic ?: existing.topic,
                nsfw = channel.nsfw ?: existing.nsfw,
                last_message_id = channel.last_message_id ?: existing.last_message_id,
                parent_id = channel.parent_id ?: existing.parent_id,
                recipients = updatedRecipients,
                recipient_ids = updatedRecipientIds,
                icon = channel.icon ?: existing.icon,
                thread_metadata = channel.thread_metadata ?: existing.thread_metadata,
                message_count = channel.message_count ?: existing.message_count,
                member_count = channel.member_count ?: existing.member_count,
                total_message_sent = channel.total_message_sent ?: existing.total_message_sent,
                available_tags = channel.available_tags ?: existing.available_tags,
                applied_tags = channel.applied_tags ?: existing.applied_tags,
                permission_overwrites = channel.permission_overwrites ?: existing.permission_overwrites,
                member_list_id = channel.member_list_id ?: existing.member_list_id,
                flags = channel.flags ?: existing.flags
            ) ?: channel.copy(
                recipients = updatedRecipients,
                recipient_ids = updatedRecipientIds
            )
            
            updated.recipients?.forEach { userStore.handleUserUpdate(it) }
            
            current + (channel.id to updated)
        }
    }

    fun removeChannel(channelId: String) {
        _channels.update { it - channelId }
    }

    fun clear() {
        _guilds.value = emptyMap()
        _channels.value = emptyMap()
    }
}
