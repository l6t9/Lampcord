package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlin.time.Clock
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.settings.Settings

class EmojiStore {
    private val maxSamples = 70
    private val minScoreThreshold = 10
    
    private var usageMap = mutableMapOf<String, List<Long>>()
    private var stickerUsageMap = mutableMapOf<String, List<Long>>()

    var frequentEmojis by mutableStateOf<List<String>>(emptyList())
        private set
    var frequentStickers by mutableStateOf<List<String>>(emptyList())
        private set

    init {
        loadUsage()
        updateFrequentEmojis()
        updateFrequentStickers()
    }

    fun onEmojiUsed(emojiKey: String) {
        val now = Clock.System.now().toEpochMilliseconds()
        val currentUsage = usageMap[emojiKey]?.toMutableList() ?: mutableListOf()
        currentUsage.add(now)
        
        // Keep only the last maxSamples
        usageMap[emojiKey] = currentUsage.takeLast(maxSamples)
        
        saveUsage()
        updateFrequentEmojis()
    }

    fun onStickerUsed(stickerId: String) {
        val now = Clock.System.now().toEpochMilliseconds()
        val currentUsage = stickerUsageMap[stickerId]?.toMutableList() ?: mutableListOf()
        currentUsage.add(now)
        
        stickerUsageMap[stickerId] = currentUsage.takeLast(maxSamples)
        
        saveUsage()
        updateFrequentStickers()
    }

    private fun updateFrequentEmojis() {
        val now = Clock.System.now().toEpochMilliseconds()
        val scores = usageMap.mapValues { (_, times) ->
            times.sumOf { time -> getWeight(getDaysDiff(time, now)) }
        }.filter { it.value > minScoreThreshold }

        val sorted = scores.entries.sortedByDescending { it.value }
            .map { it.key }
            .take(40)

        frequentEmojis = if (sorted.size < 40) {
            val defaults = listOf("❓", "🤔", "❌", "✅", "🔥", "bread", "fork_and_knife", "yum", "weary", "tired_face", "poop", "thumbsup", "100")
            (sorted + defaults).distinct().take(40)
        } else {
            sorted
        }
    }

    private fun updateFrequentStickers() {
        val now = Clock.System.now().toEpochMilliseconds()
        val scores = stickerUsageMap.mapValues { (_, times) ->
            times.sumOf { time -> getWeight(getDaysDiff(time, now)) }
        }.filter { it.value > minScoreThreshold }

        frequentStickers = scores.entries.sortedByDescending { it.value }
            .map { it.key }
            .take(40)
    }

    private fun getDaysDiff(then: Long, now: Long): Int {
        return ((now - then) / 86400000).toInt()
    }

    private fun getWeight(days: Int): Int {
        return when {
            days <= 3 -> 100
            days <= 15 -> 70
            days <= 30 -> 50
            days <= 45 -> 30
            days <= 80 -> 10
            else -> 0
        }
    }

    private fun loadUsage() {
        try {
            val json = Settings.shared.emojiUsageJson
            usageMap = Json.decodeFromString<Map<String, List<Long>>>(json).toMutableMap()
        } catch (e: Exception) {
            usageMap = mutableMapOf()
        }
        try {
            val json = Settings.shared.stickerUsageJson
            stickerUsageMap = Json.decodeFromString<Map<String, List<Long>>>(json).toMutableMap()
        } catch (e: Exception) {
            stickerUsageMap = mutableMapOf()
        }
    }

    private fun saveUsage() {
        try {
            val json = Json.encodeToString(usageMap)
            Settings.shared.emojiUsageJson = json
        } catch (e: Exception) {
            // Ignore
        }
        try {
            val json = Json.encodeToString(stickerUsageMap)
            Settings.shared.stickerUsageJson = json
        } catch (e: Exception) {
            // Ignore
        }
    }
}
