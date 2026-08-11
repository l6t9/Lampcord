package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.toTwemojiUrl
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
private data class EmojiEntry(
    val n: List<String>,
    val s: String
)

object EmojiIndex {
    private var charToNames = mapOf<String, List<String>>()
    private var nameToChar = mapOf<String, String>()
    private var allEmojis = listOf<Emoji>()
    private var initialized = false

    fun initialize() {
        if (initialized) return
        try {
            val jsonBytes = ResourceLoader.readBytes("files/emojis.json")
            val jsonText = jsonBytes?.decodeToString() ?: ""
            val entries = Json.decodeFromString<List<EmojiEntry>>(jsonText)
            
            val charMap = mutableMapOf<String, List<String>>()
            val nameMap = mutableMapOf<String, String>()
            val emojiList = mutableListOf<Emoji>()
            
            entries.forEach { entry ->
                charMap[entry.s] = entry.n
                entry.n.forEach { name ->
                    nameMap[name] = entry.s
                }
                emojiList.add(
                    Emoji(
                        id = null,
                        name = entry.n.firstOrNull() ?: entry.s,
                        url = entry.s.toTwemojiUrl()
                    )
                )
            }
            
            charToNames = charMap
            nameToChar = nameMap
            allEmojis = emojiList
            initialized = true
        } catch (e: Exception) {
            println("EmojiIndex initialization failed: ${e.message}")
        }
    }

    fun getAllEmojis(): List<Emoji> = allEmojis

    fun getCharForName(name: String): String? = nameToChar[name.removeSurrounding(":")]

    fun getNamesForChar(char: String): List<String>? = charToNames[char]

    fun getTwemojiUrl(char: String): String = char.toTwemojiUrl()
    
    fun findEmojiInString(content: String, startIndex: Int): Pair<String, Int>? {
        // Search for the longest matching emoji string starting at startIndex
        // Most emojis are 1-2 chars, but ZWJ sequences can be longer
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
}
