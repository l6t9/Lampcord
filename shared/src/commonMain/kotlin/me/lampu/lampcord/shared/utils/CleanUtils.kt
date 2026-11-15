package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.settings.Settings

object CleanUtils {
    private val WHITESPACE_RUN = Regex("\\s+")

    private val letterNormalizationMap = mapOf(
        'á' to 'a', 'à' to 'a', 'ä' to 'a', 'â' to 'a', 'ã' to 'a', 'å' to 'a', 'ą' to 'a',
        'Á' to 'A', 'À' to 'A', 'Ä' to 'A', 'Â' to 'A', 'Ã' to 'A', 'Å' to 'A', 'Ą' to 'A',
        'é' to 'e', 'è' to 'e', 'ë' to 'e', 'ê' to 'e', 'ę' to 'e',
        'É' to 'E', 'È' to 'E', 'Ë' to 'E', 'Ê' to 'E', 'Ę' to 'E',
        'í' to 'i', 'ì' to 'i', 'ï' to 'i', 'î' to 'i',
        'Í' to 'I', 'Ì' to 'I', 'Ï' to 'I', 'Î' to 'I',
        'ó' to 'o', 'ò' to 'o', 'ö' to 'o', 'ô' to 'o', 'õ' to 'o', 'ø' to 'o',
        'Ó' to 'O', 'Ò' to 'O', 'Ö' to 'O', 'Ô' to 'O', 'Õ' to 'O', 'Ø' to 'O',
        'ú' to 'u', 'ù' to 'u', 'ü' to 'u', 'û' to 'u',
        'Ú' to 'U', 'Ù' to 'U', 'Ü' to 'U', 'Û' to 'U',
        'ç' to 'c', 'Ç' to 'C', 'ć' to 'c', 'Ć' to 'C',
        'ñ' to 'n', 'Ñ' to 'N', 'ń' to 'n', 'Ń' to 'N',
        'ś' to 's', 'Ś' to 'S', 'š' to 's', 'Š' to 'S',
        'ł' to 'l', 'Ł' to 'L',
        'ż' to 'z', 'Ż' to 'Z', 'ź' to 'z', 'Ź' to 'Z', 'ž' to 'z', 'Ž' to 'Z',
        'ý' to 'y', 'Ý' to 'Y', 'ÿ' to 'y', 'Ÿ' to 'Y',
        'đ' to 'd', 'Đ' to 'D', 'ß' to 's'
    )

    private fun isEmoji(codePoint: Int): Boolean {
        return codePoint in 0x1F300..0x1FAFF ||
            codePoint in 0x2600..0x27BF ||
            codePoint in 0x1F000..0x1F02F ||
            codePoint in 0x1F0A0..0x1F0FF ||
            codePoint in 0xFE00..0xFE0F ||
            codePoint in 0x1F900..0x1F9FF ||
            codePoint in 0x1FA70..0x1FAFF
    }

    private fun isDash(codePoint: Int): Boolean {
        return codePoint == '-'.code || codePoint == 0x2010 || codePoint == 0x2011 || codePoint == 0x2012 ||
            codePoint == 0x2013 || codePoint == 0x2014 || codePoint == 0x2015 || codePoint == 0x2212
    }

    private fun codePointAt(str: String, index: Int): Int {
        val high = str[index]
        if (high.isHighSurrogate() && index + 1 < str.length) {
            val low = str[index + 1]
            if (low.isLowSurrogate()) {
                return ((high.code - 0xD800) shl 10) + (low.code - 0xDC00) + 0x10000
            }
        }
        return high.code
    }

    private fun charCount(codePoint: Int): Int = if (codePoint >= 0x10000) 2 else 1

    private fun isLetterOrDigitCp(codePoint: Int): Boolean =
        if (codePoint <= 0xFFFF) codePoint.toChar().isLetterOrDigit() else false

    fun cleanChannelName(name: String, isCategory: Boolean = false): String {
        val settings = Settings.shared
        val removeEmojis = settings.cleanChannelsRemoveEmojis
        val hideSymbols = settings.cleanChannelsHideSymbols
        val normalizeLetters = settings.cleanChannelsNormalizeLetters
        val capitalizeCategories = settings.cleanChannelsCapitalizeCategories

        if (!removeEmojis && !hideSymbols && !normalizeLetters && !(isCategory && capitalizeCategories)) {
            return name
        }

        var result = name
        if (normalizeLetters) {
            val normalized = StringBuilder(result.length)
            for (ch in result) {
                normalized.append(letterNormalizationMap[ch] ?: ch)
            }
            result = normalized.toString()
        }

        val sb = StringBuilder(result.length)
        var i = 0
        while (i < result.length) {
            val codePoint = codePointAt(result, i)
            val count = charCount(codePoint)
            val ch = result[i]
            val dash = isDash(codePoint)
            val emoji = isEmoji(codePoint)

            val keep = when {
                emoji -> !removeEmojis
                dash -> true
                hideSymbols -> isLetterOrDigitCp(codePoint) || ch == ' ' || ch == '_' || ch == '#'
                else -> true
            }

            if (keep) {
                if (dash) sb.append('-') else sb.append(result, i, i + count)
            }
            i += count
        }

        var cleaned = sb.toString().replace(WHITESPACE_RUN, " ").trim()
        if (cleaned.isEmpty()) cleaned = name.trim().ifEmpty { name }

        if (isCategory && capitalizeCategories && cleaned.isNotEmpty()) {
            cleaned = cleaned.lowercase().replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
        }

        return cleaned
    }
}
