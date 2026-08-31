package me.lampu.lampcord.shared.api

import me.lampu.lampcord.shared.settings.Settings

/**
 * CDN URL builders for guild icons and user avatars.
 */
object CdnUrls {

    fun getGuildIconUrl(guildId: String, iconHash: String?, size: Int = 1024): String? {
        if (iconHash == null) return null
        return "https://cdn.discordapp.com/icons/$guildId/$iconHash.webp?size=$size"
    }

    fun getUserAvatarUrl(userId: String, avatarHash: String?, size: Int = 1024): String {
        if (avatarHash == null) return getDefaultAvatarUrl(userId)
        val extension = if (avatarHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "png"
        return "https://cdn.discordapp.com/avatars/$userId/$avatarHash.$extension?size=$size"
    }

    fun getDefaultAvatarUrl(userId: String): String {
        val index = ((userId.toLongOrNull() ?: 0L) shr 22) % 6
        return "https://cdn.discordapp.com/embed/avatars/$index.png"
    }
}
