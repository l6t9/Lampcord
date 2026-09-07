package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.Logging

class UserStore {
    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser.asStateFlow()

    private val _users = MutableStateFlow<Map<String, User>>(emptyMap())
    val users: StateFlow<Map<String, User>> = _users.asStateFlow()

    private val _members = MutableStateFlow<Map<String, Map<String, Member>>>(emptyMap())
    val members: StateFlow<Map<String, Map<String, Member>>> = _members.asStateFlow()

    fun setCurrentUser(user: User?, raw: kotlinx.serialization.json.JsonObject? = null) {
        Logging.i("UserStore", "Setting current user: ${user?.username} (${user?.id})")
        _currentUser.value = user
        if (user != null) handleUserUpdate(user, raw)
    }

    fun handleUserUpdate(user: User, raw: kotlinx.serialization.json.JsonObject? = null) {
        Logging.d("UserStore", "Updating user: ${user.username} (${user.id})")
        _users.update { current ->
            val existing = current[user.id]
            val updated = existing?.copy(
                username = user.username ?: existing.username,
                global_name = user.global_name ?: existing.global_name,
                avatar = user.avatar ?: existing.avatar,
                avatar_decoration_data = if (raw?.containsKey("avatar_decoration_data") == true) user.avatar_decoration_data else (user.avatar_decoration_data ?: existing.avatar_decoration_data),
                discriminator = user.discriminator ?: existing.discriminator,
                public_flags = user.public_flags ?: existing.public_flags,
                flags = user.flags ?: existing.flags,
                accent_color = user.accent_color ?: existing.accent_color,
                banner = user.banner ?: existing.banner,
                bio = user.bio ?: existing.bio,
                pronouns = user.pronouns ?: existing.pronouns,
                display_name_styles = if (raw?.containsKey("display_name_styles") == true) user.display_name_styles else (user.display_name_styles ?: existing.display_name_styles)
            )
                ?: user
            current + (user.id to updated)
        }
        
        if (_currentUser.value?.id == user.id) {
            _currentUser.update { current ->
                val userInMap = _users.value[user.id]
                userInMap ?: current
            }
        }
    }

    fun cacheMember(guildId: String, userId: String, member: Member, raw: kotlinx.serialization.json.JsonObject? = null) {
        _members.update { current ->
            val guildMembers = current[guildId]?.toMutableMap() ?: mutableMapOf()
            val existing = guildMembers[userId]
            
            val updatedMember = existing?.copy(
                user = member.user ?: existing.user,
                nick = member.nick ?: existing.nick,
                avatar = member.avatar ?: existing.avatar,
                roles = member.roles.ifEmpty { existing.roles },
                display_name_styles = if (raw?.containsKey("display_name_styles") == true) member.display_name_styles else (member.display_name_styles ?: existing.display_name_styles),
                premium_since = member.premium_since ?: existing.premium_since,
                pending = member.pending ?: existing.pending,
                permissions = member.permissions ?: existing.permissions,
                communication_disabled_until = member.communication_disabled_until ?: existing.communication_disabled_until,
                deaf = member.deaf,
                mute = member.mute,
                flags = member.flags,
                presence = member.presence ?: existing.presence
            )
                ?: member
            
            guildMembers[userId] = updatedMember
            current + (guildId to guildMembers)
        }
        
        member.user?.let { handleUserUpdate(it, raw) }
    }

    fun getUser(userId: String): User? = _users.value[userId]

    fun getMember(guildId: String, userId: String): Member? {
        return _members.value[guildId]?.get(userId)
    }

    fun clear() {
        _users.value = emptyMap()
        _members.value = emptyMap()
        _currentUser.value = null
    }

    fun getCurrentMember(guildId: String?): Member? {
        val userId = _currentUser.value?.id ?: return null
        if (guildId == null) return null
        return getMember(guildId, userId)
    }
}
