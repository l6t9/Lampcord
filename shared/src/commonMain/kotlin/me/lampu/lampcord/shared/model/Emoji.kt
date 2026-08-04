package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
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
    val url: String? = null
)

fun Emoji.getDisplayUrl(): String? {
    if (url != null) return url
    if (id != null) {
        val ext = if (animated == true) "gif" else "png"
        return "https://cdn.discordapp.com/emojis/$id.$ext?size=48"
    }
    return name?.let { EmojiIndex.getTwemojiUrl(it) } ?: name?.toTwemojiUrl()
}

fun String.toTwemojiUrl(): String {
    val codepoints = mutableListOf<String>()
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
        // Variation Selector-16 (U+FE0F) is stripped in Twemoji filenames
        // but only if it's not part of a sequence that requires it?
        // Actually, Twemoji's standard is to strip all FE0F.
        if (c1.code == 0xFE0F) {
            i++
            continue
        }
        codepoints.add(c1.code.toString(16).lowercase())
        i++
    }
    if (codepoints.isEmpty()) return ""
    val codepointStr = codepoints.joinToString("-")
    return "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/72x72/$codepointStr.png"
}
