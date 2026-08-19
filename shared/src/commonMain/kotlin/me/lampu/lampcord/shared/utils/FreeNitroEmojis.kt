package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.settings.Settings

object FreeNitroEmojis {
    val emojiRegex = Regex("""<(a)?:(F_)?([a-zA-Z0-9_]+):(\d+)>""")
    val markdownRegexCompound = Regex(
        """(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/emojis/(\d+)\.(gif|png|webp)(?:\?[^)\s]*)?)\)?"""
    )
    val markdownRegexSingle = Regex(
        """^(?:\[(?:[a-zA-Z0-9_~]+|\u2236[a-zA-Z0-9_~]+\u2236)]\()?(https://cdn\.discordapp\.com/emojis/(\d+)\.(gif|png|webp)(?:\?[^)\s]*)?)\)?$"""
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

            val ext = if (animated) "gif" else "png"
            val url = "https://cdn.discordapp.com/emojis/$id.$ext?size=48&name=$name"
            
            // Supporting the Aliucord format types (defaulting to standard markdown)
            return "[$name]($url)"
        }

        return if (emoji.id != null) {
            "<${if (emoji.animated == true) "a" else ""}:${emoji.name}:${emoji.id}>"
        } else {
            emoji.name ?: ""
        }
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
            val useWebp = settings.useWebpEmojis

            val urlBuilder = StringBuilder("https://cdn.discordapp.com/emojis/$emojiId")
            if (useWebp) {
                urlBuilder.append(".webp?name=$emojiName&lossless=true")
                if (isAnimated) urlBuilder.append("&animated=true")
            } else {
                urlBuilder.append(if (isAnimated) ".gif" else ".png")
                urlBuilder.append("?name=$emojiName")
            }
            urlBuilder.append("&size=48")

            "[$emojiName]($urlBuilder)"
        }
    }

    fun preprocessIncoming(message: Message): Message {
        val settings = Settings.shared
        if (!settings.freeNitroEmojis || !settings.realmojis) return message

        val content = message.content
        val embeds = message.embeds.toMutableList()
        val regex = if (settings.compoundRealmojis) markdownRegexCompound else markdownRegexSingle

        var changed = false
        val newContent = regex.replace(content) { match ->
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

        return if (changed) message.copy(content = newContent, embeds = embeds) else message
    }
}
