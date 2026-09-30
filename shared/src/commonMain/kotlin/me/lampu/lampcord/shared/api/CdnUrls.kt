package me.lampu.lampcord.shared.api

import me.lampu.lampcord.shared.settings.Settings

object CdnUrls {

    fun normalizeSize(size: Int): Int = when {
        size <= 16 -> 16
        size <= 32 -> 32
        size <= 64 -> 64
        size <= 128 -> 128
        size <= 256 -> 256
        size <= 512 -> 512
        else -> 1024
    }

    fun getGuildIconUrl(guildId: String, iconHash: String?, size: Int = 256): String? {
        if (iconHash == null) return null
        val validSize = normalizeSize(size)
        val extension = if (iconHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "png"
        return "https://cdn.discordapp.com/icons/$guildId/$iconHash.$extension?size=$validSize"
    }

    fun getUserAvatarUrl(userId: String, avatarHash: String?, size: Int = 256): String {
        if (avatarHash == null) return getDefaultAvatarUrl(userId)
        val validSize = normalizeSize(size)
        val extension = if (avatarHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "png"
        return "https://cdn.discordapp.com/avatars/$userId/$avatarHash.$extension?size=$validSize"
    }

    fun getMemberAvatarUrl(guildId: String, userId: String, memberAvatarHash: String?, userAvatarHash: String?, size: Int = 256): String {
        val validSize = normalizeSize(size)
        if (memberAvatarHash != null) {
            val extension = if (memberAvatarHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "png"
            return "https://cdn.discordapp.com/guilds/$guildId/users/$userId/avatars/$memberAvatarHash.$extension?size=$validSize"
        }
        return getUserAvatarUrl(userId, userAvatarHash, validSize)
    }

    fun getDefaultAvatarUrl(userId: String): String {
        val index = ((userId.toLongOrNull() ?: 0L) shr 22) % 6
        return "https://cdn.discordapp.com/embed/avatars/$index.png"
    }

    fun getAvatarDecorationUrl(asset: String?, size: Int = 256): String? {
        if (asset == null) return null
        val validSize = normalizeSize(size)
        return "https://cdn.discordapp.com/avatar-decoration-presets/$asset.png?size=$validSize&passthrough=true"
    }

    fun getChannelIconUrl(channelId: String, iconHash: String?, size: Int = 256): String? {
        if (iconHash == null) return null
        val validSize = normalizeSize(size)
        val extension = if (iconHash.startsWith("a_") && !Settings.shared.reduceMotion) "gif" else "png"
        return "https://cdn.discordapp.com/channel-icons/$channelId/$iconHash.$extension?size=$validSize"
    }
}
