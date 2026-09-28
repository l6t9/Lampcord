package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.toTwemojiUrl
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

object EmojiIndex {
    private var charToNames = mapOf<String, List<String>>()
    private var nameToChar = mapOf<String, String>()
    private var categorizedEmojis = mapOf<String, List<Emoji>>()
    private var allEmojis = listOf<Emoji>()
    private var initialized = false

    private val json = Json {
        ignoreUnknownKeys = true
    }

    fun initialize() {
        if (initialized) return
        try {
            val jsonBytes = ResourceLoader.readBytes("files/emojis.json")
            val jsonText = jsonBytes?.decodeToString() ?: ""
            val categories = json.decodeFromString<Map<String, List<EmojiEntry>>>(jsonText)

            val charMap = mutableMapOf<String, List<String>>()
            val nameMap = mutableMapOf<String, String>()
            val emojiList = mutableListOf<Emoji>()
            val categorized = mutableMapOf<String, List<Emoji>>()

            categories.forEach { (category, entries) ->
                val categoryEmojis = mutableListOf<Emoji>()
                entries.forEach { entry ->
                    charMap[entry.surrogates] = entry.names
                    entry.names.forEach { name ->
                        nameMap[name] = entry.surrogates
                    }
                    val emoji = Emoji(
                        id = null,
                        name = entry.surrogates,
                        url = entry.surrogates.toTwemojiUrl()
                    )
                    categoryEmojis.add(emoji)
                    emojiList.add(emoji)
                }
                categorized[category] = categoryEmojis
            }

            charToNames = charMap
            nameToChar = nameMap
            allEmojis = emojiList
            categorizedEmojis = categorized
            initialized = true
        } catch (e: Exception) {
            println("EmojiIndex initialization failed: ${e.message}")
        }
    }

    fun getAllEmojis(): List<Emoji> = allEmojis

    fun getCategorizedEmojis(): Map<String, List<Emoji>> = categorizedEmojis

    fun getCharForName(name: String): String? = nameToChar[name.removeSurrounding(":")]

    fun getNamesForChar(char: String): List<String>? = charToNames[char]

    fun getTwemojiUrl(char: String): String = char.toTwemojiUrl()

    fun findEmojiInString(content: String, startIndex: Int): Pair<String, Int>? {
        for (len in 10 downTo 1) {
            if (startIndex + len <= content.length) {
                val sub = content.substring(startIndex, startIndex + len)
                if (charToNames.containsKey(sub)) {
                    return sub to len
                }
                // Also check without trailing FE0F
                if (sub.endsWith('\uFE0F')) {
                    val subNoVar = sub.dropLast(1)
                    if (charToNames.containsKey(subNoVar)) {
                        return subNoVar to len
                    }
                }
            }
        }
        return null
    }

    @Serializable
    data class EmojiEntry(
        val names: List<String>,
        val surrogates: String,
        val unicodeVersion: Double
    )
}
