package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.settings.Settings

object CleanUtils {
    private val letterNormalizationMap = mapOf(
        'á' to 'a', 'é' to 'e', 'í' to 'i', 'ó' to 'o', 'ú' to 'u',
        'Á' to 'A', 'É' to 'E', 'Í' to 'I', 'Ó' to 'O', 'Ú' to 'U',
        'à' to 'a', 'è' to 'e', 'ì' to 'i', 'ò' to 'o', 'ù' to 'u',
        'À' to 'A', 'È' to 'E', 'Ì' to 'I', 'Ò' to 'O', 'Ù' to 'U',
        'ä' to 'a', 'ë' to 'e', 'ï' to 'i', 'ö' to 'o', 'ü' to 'u',
        'Ä' to 'A', 'Ë' to 'E', 'Ï' to 'I', 'Ö' to 'O', 'Ü' to 'U',
        'â' to 'a', 'ê' to 'e', 'î' to 'i', 'ô' to 'o', 'û' to 'u',
        'Â' to 'A', 'Ê' to 'E', 'Î' to 'I', 'Ô' to 'O', 'Û' to 'U',
        'ç' to 'c', 'Ç' to 'C',
        'ñ' to 'n', 'Ñ' to 'N',
        'ø' to 'o', 'å' to 'a', 'Ø' to 'O', 'Å' to 'A',
        'ś' to 's', 'Ś' to 'S', 'ł' to 'l', 'Ł' to 'L',
        'ż' to 'z', 'Ż' to 'Z', 'ć' to 'c', 'Ć' to 'C',
        'ę' to 'e', 'Ę' to 'E', 'ó' to 'o', 'Ó' to 'O',
        'ś' to 's', 'Ś' to 'S', 'ń' to 'n', 'Ń' to 'N',
        'ż' to 'z', 'Ż' to 'Z', 'ą' to 'a', 'Ą' to 'A',
        'ś' to 's', 'Ś' to 'S', 'ł' to 'l', 'Ł' to 'L',
        'ć' to 'c', 'Ć' to 'C', 'ź' to 'z', 'Ź' to 'Z',
        'ę' to 'e', 'Ę' to 'E', 'ó' to 'o', 'Ó' to 'O'
    )

    fun cleanChannelName(name: String, isCategory: Boolean = false): String {
        val settings = Settings.shared
        var result = name

        if (settings.cleanChannelsNormalizeLetters) {
            val normalized = StringBuilder()
            for (ch in result) {
                normalized.append(letterNormalizationMap.getOrDefault(ch, ch))
            }
            result = normalized.toString()
        }

        val sb = StringBuilder()
        var i = 0
        while (i < result.length) {
            val ch = result[i]
            val shouldKeep = when {
                ch.isLetterOrDigit() -> true
                ch == '-' || ch == ' ' -> true
                else -> true
            }

            if (shouldKeep) sb.append(ch)
            i++
        }

        val normalized = sb.toString().replace(Regex("\\s+"), " ").trim()

        if (isCategory && settings.cleanChannelsCapitalizeCategories && normalized.isNotEmpty()) {
            result = normalized.lowercase().replaceFirstChar { it.uppercase() }
        } else {
            result = normalized
        }

        return result.ifEmpty { name }
    }
}