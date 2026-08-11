package me.lampu.lampcord.shared.state

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import me.lampu.lampcord.shared.model.*

class UserStore {
    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser.asStateFlow()

    private val _users = MutableStateFlow<Map<String, User>>(emptyMap())
    val users: StateFlow<Map<String, User>> = _users.asStateFlow()

    private val _members = MutableStateFlow<Map<String, Map<String, Member>>>(emptyMap())
    val members: StateFlow<Map<String, Map<String, Member>>> = _members.asStateFlow()

    fun setCurrentUser(user: User?) {
        _currentUser.value = user
        if (user != null) handleUserUpdate(user)
    }

    fun handleUserUpdate(user: User) {
        val currentUsers = _users.value.toMutableMap()
        val existing = currentUsers[user.id]
        
        val updatedUser = if (existing == null) {
            user
        } else {
            existing.copy(
                username = user.username ?: existing.username,
                global_name = user.global_name ?: existing.global_name,
                avatar = user.avatar ?: existing.avatar,
                avatar_decoration_data = user.avatar_decoration_data ?: existing.avatar_decoration_data,
                discriminator = user.discriminator ?: existing.discriminator,
                public_flags = user.public_flags ?: existing.public_flags,
                flags = user.flags ?: existing.flags,
                accent_color = user.accent_color ?: existing.accent_color,
                banner = user.banner ?: existing.banner,
                bio = user.bio ?: existing.bio,
                pronouns = user.pronouns ?: existing.pronouns,
                display_name_styles = user.display_name_styles ?: existing.display_name_styles
            )
        }
        
        currentUsers[user.id] = updatedUser
        _users.value = currentUsers
        
        if (_currentUser.value?.id == user.id) {
            _currentUser.value = updatedUser
        }
    }

    fun cacheMember(guildId: String, userId: String, member: Member) {
        val currentMembers = _members.value.toMutableMap()
        val guildMembers = currentMembers[guildId]?.toMutableMap() ?: mutableMapOf()
        
        val existing = guildMembers[userId]
        val updatedMember = if (existing == null) {
            member
        } else {
            existing.copy(
                user = member.user ?: existing.user,
                nick = member.nick ?: existing.nick,
                avatar = member.avatar ?: existing.avatar,
                roles = if (member.roles.isNotEmpty()) member.roles else existing.roles,
                display_name_styles = member.display_name_styles ?: existing.display_name_styles
            )
        }
        
        guildMembers[userId] = updatedMember
        currentMembers[guildId] = guildMembers
        _members.value = currentMembers
        
        member.user?.let { handleUserUpdate(it) }
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
