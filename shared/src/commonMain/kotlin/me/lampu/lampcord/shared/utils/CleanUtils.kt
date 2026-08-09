package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.settings.Settings

object CleanUtils {
    fun cleanChannelName(name: String, isCategory: Boolean = false): String {
        val settings = Settings.shared
        var result = name

        if (settings.cleanChannelsNormalizeLetters) {
            // Basic normalization (KMP compatible)
            // KMP doesn't have java.text.Normalizer, so we do a simple replacement for common cases if needed
            // But for now, we'll just keep it as is or implement a simple mapping
        }

        val sb = StringBuilder()
        var i = 0
        while (i < result.length) {
            val cp = result[i].code
            val isEmoji = (cp in 0x1F300..0x1F9FF) || (cp in 0x2600..0x27BF)
            val isSymbol = !result[i].isLetterOrDigit() && result[i] != '-' && result[i] != ' '

            val shouldKeep = when {
                isEmoji -> !settings.cleanChannelsRemoveEmojis
                isSymbol -> !settings.cleanChannelsHideSymbols
                else -> true
            }

            if (shouldKeep) sb.append(result[i])
            i++
        }

        result = sb.toString().replace(Regex("\\s+"), " ").trim()

        if (isCategory && settings.cleanChannelsCapitalizeCategories && result.isNotEmpty()) {
            result = result.lowercase().replaceFirstChar { it.uppercase() }
        }

        return result.ifEmpty { name }
    }
}
