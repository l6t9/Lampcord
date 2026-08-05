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
        userCache[user.id] = user
    }

    fun getUser(userId: String): User? = userCache[userId]

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
                roles = if (member.roles.isNotEmpty()) member.roles else existing.roles
            )
        }
    }

    fun getMember(guildId: String, userId: String): Member? {
        return memberCache[guildId]?.get(userId)
    }
}
