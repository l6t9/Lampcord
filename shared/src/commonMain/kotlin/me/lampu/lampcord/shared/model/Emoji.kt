package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.EmojiIndex

@Serializable
data class Emoji(
    val id: String? = null,
    val name: String? = null,
    val roles: List<String>? = null,
    val user: User? = null,
    val require_colons: Boolean? = null,
    val managed: Boolean? = null,
    val animated: Boolean? = null,
    val available: Boolean? = null,
    val url: String? = null,
    val guild_id: String? = null
)

fun Emoji.getDisplayUrl(): String? {
    if (url != null) return url
    if (id != null) {
        val ext = if (animated == true && !Settings.shared.reduceMotion) "gif" else "png"
        return "https://cdn.discordapp.com/emojis/$id.$ext?size=48"
    }
    val unicode = name?.let { EmojiIndex.getCharForName(it) } ?: name
    return unicode?.toTwemojiUrl()
}

internal const val TWEMOJI_CDN_BASE_URL = "https://cdn.jsdelivr.net/gh/jdecked/twemoji@v17.0.3/assets/72x72"

fun String.toTwemojiUrl(): String {
    val codepoints = mutableListOf<String>()
    val hasJoiner = contains('\u200D')
    var i = 0
    while (i < length) {
        val c1 = this[i]
        if (c1.isHighSurrogate() && i + 1 < length) {
            val c2 = this[i + 1]
            if (c2.isLowSurrogate()) {
                val codepoint = (c1.code - 0xD800 shl 10) + (c2.code - 0xDC00) + 0x10000
                codepoints.add(codepoint.toString(16).lowercase())
                i += 2
                continue
            }
        }
        if (c1.code == 0xFE0F && !hasJoiner) {
            i++
            continue
        }
        codepoints.add(c1.code.toString(16).lowercase())
        i++
    }
    if (codepoints.isEmpty()) return ""
    // Twemoji's eye-in-speech-bubble asset omits both variation selectors.
    val codepointStr = codepoints.joinToString("-").replace(
        "1f441-fe0f-200d-1f5e8-fe0f",
        "1f441-200d-1f5e8"
    )
    return "$TWEMOJI_CDN_BASE_URL/$codepointStr.png"
}
