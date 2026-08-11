package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.model.*

class EntityStore {
    private val _users = MutableStateFlow<Map<String, User>>(emptyMap())
    val users: StateFlow<Map<String, User>> = _users.asStateFlow()

    private val _guilds = MutableStateFlow<Map<String, Guild>>(emptyMap())
    val guilds: StateFlow<Map<String, Guild>> = _guilds.asStateFlow()

    private val _channels = MutableStateFlow<Map<String, Channel>>(emptyMap())
    val channels: StateFlow<Map<String, Channel>> = _channels.asStateFlow()

    private val _members = MutableStateFlow<Map<String, Map<String, Member>>>(emptyMap())
    val members: StateFlow<Map<String, Map<String, Member>>> = _members.asStateFlow()

    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser.asStateFlow()

    fun updateCurrentUser(user: User) {
        _currentUser.value = user
        updateUser(user)
    }

    fun updateUser(user: User) {
        _users.update { current ->
            val existing = current[user.id]
            val updated = existing?.copy(
                username = user.username ?: existing.username,
                global_name = user.global_name ?: existing.global_name,
                avatar = user.avatar ?: existing.avatar,
                discriminator = user.discriminator ?: existing.discriminator,
                public_flags = user.public_flags ?: existing.public_flags,
                flags = user.flags ?: existing.flags,
                accent_color = user.accent_color ?: existing.accent_color,
                banner = user.banner ?: existing.banner,
                bio = user.bio ?: existing.bio,
                pronouns = user.pronouns ?: existing.pronouns,
                display_name_styles = user.display_name_styles ?: existing.display_name_styles
            )
                ?: user
            current + (user.id to updated)
        }
        
        if (_currentUser.value?.id == user.id) {
            _currentUser.value = _users.value[user.id]
        }
    }

    fun updateGuild(guild: Guild) {
        _guilds.update { it + (guild.id to guild) }
        guild.channels?.forEach { updateChannel(it) }
    }

    fun removeGuild(guildId: String) {
        _guilds.update { it - guildId }
        _members.update { it - guildId }
        _channels.update { current ->
            current.filterValues { it.guild_id != guildId }
        }
    }

    fun updateChannel(channel: Channel) {
        _channels.update { it + (channel.id to channel) }
    }

    fun removeChannel(channelId: String) {
        _channels.update { it - channelId }
    }

    fun updateMember(guildId: String, member: Member) {
        val userId = member.user?.id ?: return
        _members.update { current ->
            val guildMembers = current[guildId] ?: emptyMap()
            val existing = guildMembers[userId]
            val updated = existing?.copy(
                user = member.user,
                nick = member.nick ?: existing.nick,
                avatar = member.avatar ?: existing.avatar,
                roles = member.roles.ifEmpty { existing.roles },
                display_name_styles = member.display_name_styles ?: existing.display_name_styles
            )
                ?: member
            current + (guildId to (guildMembers + (userId to updated)))
        }
        updateUser(member.user)
    }

    fun clear() {
        _users.value = emptyMap()
        _guilds.value = emptyMap()
        _channels.value = emptyMap()
        _members.value = emptyMap()
        _currentUser.value = null
    }
}
