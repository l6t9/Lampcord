package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.SnapshotStateMap
import me.lampu.lampcord.shared.model.*

class UserStore {
    var currentUser by mutableStateOf<User?>(null)
    
    // User Cache: userId -> User
    private val userCache = mutableStateMapOf<String, User>()
    
    // Member Cache: guildId -> userId -> Member
    private val memberCache = mutableStateMapOf<String, SnapshotStateMap<String, Member>>()

    fun cacheUser(user: User) {
        handleUserUpdate(user)
    }

    fun getUser(userId: String): User? = userCache[userId]

    fun handleUserUpdate(user: User) {
        val existing = userCache[user.id]
        if (existing == null) {
            userCache[user.id] = user
        } else {
            userCache[user.id] = existing.copy(
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
        if (currentUser?.id == user.id) {
            currentUser = userCache[user.id]
        }
    }

    fun cacheMember(guildId: String, userId: String, member: Member) {
        val guildMap = memberCache.getOrPut(guildId) { mutableStateMapOf() }
        val existing = guildMap[userId]
        if (existing == null) {
            guildMap[userId] = member
        } else {
            guildMap[userId] = existing.copy(
                user = member.user ?: existing.user,
                nick = member.nick ?: existing.nick,
                avatar = member.avatar ?: existing.avatar,
                roles = if (member.roles.isNotEmpty()) member.roles else existing.roles,
                display_name_styles = member.display_name_styles ?: existing.display_name_styles
            )
        }
        member.user?.let { handleUserUpdate(it) }
    }

    fun getMember(guildId: String, userId: String): Member? {
        return memberCache[guildId]?.get(userId)
    }

    fun clear() {
        userCache.clear()
        memberCache.clear()
        currentUser = null
    }
}
