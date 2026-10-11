package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.Sticker
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.model.customEmojiCdnUrl
import me.lampu.lampcord.shared.settings.Settings

object FreeNitroEmojis {
    val emojiRegex = Regex("""<(a)?:(F_)?([a-zA-Z0-9_]+):(\d+)>""")
    val markdownRegexCompound = Regex(
        """(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/emojis/(\d+)\.(gif|png|webp)(?:\?[^)\s]*)?)\)?"""
    )
    val markdownRegexSingle = Regex(
        """^(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/emojis/(\d+)\.(gif|png|webp)(?:\?[^)\s]*)?)\)?$"""
    )
    val stickerRegex = Regex(
        """(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/stickers/(\d+)\.(gif|png|webp|json)(?:\?[^)\s]*)?)\)?"""
    )

    fun getReplacement(emoji: Emoji, currentUser: User?, selectedGuildId: String?): String {
        val settings = Settings.shared
        if (!settings.freeNitroEmojis) return if (emoji.id != null) "<${if (emoji.animated == true) "a" else ""}:${emoji.name}:${emoji.id}>" else emoji.name ?: ""

        val isExternal = emoji.guild_id != null && emoji.guild_id != selectedGuildId
        val needsNitro = isExternal || emoji.animated == true
        val hasNitro = (currentUser?.premium_type ?: 0) > 0

        if (needsNitro && !hasNitro) {
            val id = emoji.id ?: return emoji.name ?: ""
            val name = emoji.name ?: "emoji"
            val animated = emoji.animated == true

            if (settings.realmojis) {
                return "<${if (animated) "a" else ""}:F_$name:$id>"
            }

            val url = "${customEmojiCdnUrl(id, animated, size = 48)}&name=$name"
            
            // Supporting the Aliucord format types (defaulting to standard markdown)
            return "[$name]($url)"
        }

        return if (emoji.id != null) {
            "<${if (emoji.animated == true) "a" else ""}:${emoji.name}:${emoji.id}>"
        } else {
            emoji.name ?: ""
        }
    }

    fun getStickerReplacement(sticker: Sticker, currentUser: User?, selectedGuildId: String?): String? {
        val settings = Settings.shared
        if (!settings.freeNitroEmojis) return null

        val isExternal = sticker.guild_id != null && sticker.guild_id != selectedGuildId
        val isAnimated = sticker.format_type != 1
        val isUnavailable = sticker.available == false
        val needsNitro = isExternal || isAnimated || isUnavailable
        val hasNitro = (currentUser?.premium_type ?: 0) > 0

        if (needsNitro && !hasNitro) {
            return when (sticker.format_type) {
                4 -> "https://cdn.discordapp.com/stickers/${sticker.id}.gif?size=320"
                else -> "https://cdn.discordapp.com/stickers/${sticker.id}.png?size=320"
            }
        }
        return null
    }

    fun transformOutgoing(content: String): String {
        val settings = Settings.shared
        if (!settings.freeNitroEmojis || !settings.realmojis) return content

        return emojiRegex.replace(content) { match ->
            val isFake = match.groupValues[2] == "F_"
            if (!isFake) return@replace match.value

            val isAnimated = match.groupValues[1].isNotEmpty()
            val emojiName = match.groupValues[3]
            val emojiId = match.groupValues[4]

            val urlBuilder = StringBuilder(customEmojiCdnUrl(emojiId, isAnimated, size = 48))
            urlBuilder.append("&name=$emojiName")

            "[$emojiName]($urlBuilder)"
        }
    }

    fun preprocessIncoming(message: Message): Message {
        val settings = Settings.shared
        if (!settings.freeNitroEmojis || !settings.realmojis) return message

        var content = message.content
        val embeds = message.embeds.toMutableList()
        val stickerItems = message.sticker_items?.toMutableList() ?: mutableListOf()
        val regex = if (settings.compoundRealmojis) markdownRegexCompound else markdownRegexSingle

        var changed = false
        content = regex.replace(content) { match ->
            val url = match.groupValues[1]
            val id = match.groupValues[2]
            val ext = match.groupValues[3]

            var animated = if (ext == "gif") "a" else ""
            var name = "emoji"

            url.substringAfter('?', "").split('&').forEach { param ->
                val pair = param.split('=')
                when {
                    ext == "webp" && pair.getOrNull(0) == "animated" && pair.getOrNull(1) == "true" -> {
                        animated = "a"
                    }
                    pair.getOrNull(0) == "name" -> {
                        name = pair.getOrNull(1)?.takeWhile { it.isLetterOrDigit() || it == '_' } ?: name
                    }
                }
            }

            embeds.removeAll { embed ->
                val embedUrl = embed.url ?: embed.thumbnail?.url ?: embed.image?.url ?: 
                              embed.thumbnail?.proxy_url ?: embed.image?.proxy_url
                embedUrl?.startsWith("https://cdn.discordapp.com/emojis/$id.") == true
            }
            
            changed = true
            "<$animated:$name:$id>"
        }

        content = stickerRegex.replace(content) { match ->
            val id = match.groupValues[2]
            val ext = match.groupValues[3]
            
            val formatType = when(ext) {
                "gif" -> 4
                "json" -> 3
                "webp" -> 2
                else -> 1
            }

            if (!stickerItems.any { it.id == id }) {
                stickerItems.add(me.lampu.lampcord.shared.model.StickerItem(
                    id = id,
                    name = "sticker",
                    format_type = formatType
                ))
            }

            embeds.removeAll { embed ->
                val embedUrl = embed.url ?: embed.thumbnail?.url ?: embed.image?.url ?: 
                              embed.thumbnail?.proxy_url ?: embed.image?.proxy_url
                embedUrl?.contains("/stickers/$id.") == true
            }

            changed = true
            "" 
        }

        return if (changed) message.copy(content = content.trim(), embeds = embeds, sticker_items = stickerItems.ifEmpty { null }) else message
    }
}
