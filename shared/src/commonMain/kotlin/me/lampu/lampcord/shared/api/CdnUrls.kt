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
        val extension = if (avatarHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "webp"
        return "https://cdn.discordapp.com/avatars/$userId/$avatarHash.$extension?size=$size"
    }

    fun getMemberAvatarUrl(guildId: String, userId: String, memberAvatarHash: String?, userAvatarHash: String?, size: Int = 1024): String {
        if (memberAvatarHash != null) {
            val extension = if (memberAvatarHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "webp"
            return "https://cdn.discordapp.com/guilds/$guildId/users/$userId/avatars/$memberAvatarHash.$extension?size=$size"
        }
        return getUserAvatarUrl(userId, userAvatarHash, size)
    }

    fun getDefaultAvatarUrl(userId: String): String {
        val index = ((userId.toLongOrNull() ?: 0L) shr 22) % 6
        return "https://cdn.discordapp.com/embed/avatars/$index.png"
    }

    fun getAvatarDecorationUrl(asset: String?, size: Int = 480): String? {
        if (asset == null) return null
        // Always use the APNG passthrough variant. passthrough=false returns separate
        // promo artwork that does not match the animation's colors. Static rendering is
        // handled by loading this same APNG without animation (first frame).
        return "https://cdn.discordapp.com/avatar-decoration-presets/$asset.png?size=$size&passthrough=true"
    }
}
