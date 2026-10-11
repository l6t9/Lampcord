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

// Discord's own client builds custom emoji URLs as /emojis/{id}.{gif|webp}?size={n}&quality=lossless
// (ModelEmojiCustom.setCdnUri). The extension carries the animation flag; there is no `animated=`
// query parameter. `webp` is only downgraded to `png` on API 28/29, where animated WebP decoding
// was still unreliable.
fun customEmojiCdnUrl(id: String, animated: Boolean, size: Int = 64): String {
    val ext = if (animated) "gif" else "webp"
    return "https://cdn.discordapp.com/emojis/$id.$ext?size=$size&quality=lossless"
}

fun isUnicodeEmoji(str: String): Boolean {
    if (str.isEmpty()) return false
    if (EmojiIndex.getNamesForChar(str) != null) return true
    for (i in str.indices) {
        val char = str[i]
        if (char.code > 127 || char.code == 0x20E3 || char.code == 0xFE0F) return true
    }
    return false
}

fun Emoji.getDisplayUrl(): String? {
    if (url != null) return url
    if (id != null) {
        // Reduce Motion asks for the static frame, which the plain webp variant already is.
        return customEmojiCdnUrl(id, animated == true && !Settings.shared.reduceMotion)
    }
    val nameStr = name ?: return null
    val unicode = EmojiIndex.getCharForName(nameStr) ?: nameStr
    if (isUnicodeEmoji(unicode)) {
        return unicode.toTwemojiUrl().ifEmpty { null }
    }
    return null
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
    val codepointStr = codepoints.joinToString("-").replace(
        "1f441-fe0f-200d-1f5e8-fe0f",
        "1f441-200d-1f5e8"
    )
    return "$TWEMOJI_CDN_BASE_URL/$codepointStr.png"
}
