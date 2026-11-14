package me.lampu.lampcord.shared.state

import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import me.lampu.lampcord.shared.model.Message
import kotlin.time.Clock
import kotlin.time.Duration.Companion.days

private val HAS_CHOICES = listOf("link", "embed", "file", "image", "video", "sound", "sticker", "poll")

enum class SearchFilterKind { USER, CHANNEL, CHOICE, DATE }

data class SearchFilterDef(
    val key: String,
    val label: String,
    val kind: SearchFilterKind,
    val choices: List<String> = emptyList(),
)

val SEARCH_FILTER_DEFS: List<SearchFilterDef> = listOf(
    SearchFilterDef("from", "From a user", SearchFilterKind.USER),
    SearchFilterDef("mentions", "Mentions a user", SearchFilterKind.USER),
    SearchFilterDef("in", "In a channel", SearchFilterKind.CHANNEL),
    SearchFilterDef("has", "Has attachment or link", SearchFilterKind.CHOICE, HAS_CHOICES),
    SearchFilterDef("exclude", "Doesn't have", SearchFilterKind.CHOICE, HAS_CHOICES),
    SearchFilterDef("authortype", "Author type", SearchFilterKind.CHOICE, listOf("user", "bot", "webhook")),
    SearchFilterDef("before", "Sent before date", SearchFilterKind.DATE),
    SearchFilterDef("during", "Sent on date", SearchFilterKind.DATE),
    SearchFilterDef("after", "Sent after date", SearchFilterKind.DATE),
    SearchFilterDef("sort", "Sort order", SearchFilterKind.CHOICE, listOf("new", "old")),
)

private val SEARCH_KEY_ALIASES = mapOf(
    "from" to "from",
    "mentions" to "mentions",
    "in" to "in",
    "has" to "has",
    "exclude" to "exclude",
    "authortype" to "authortype",
    "author_type" to "authortype",
    "author" to "authortype",
    "before" to "before",
    "during" to "during",
    "after" to "after",
    "sort" to "sort",
)

fun canonicalSearchKey(raw: String): String? = SEARCH_KEY_ALIASES[raw.lowercase()]

fun searchFilterDef(key: String): SearchFilterDef? = SEARCH_FILTER_DEFS.firstOrNull { it.key == key }

data class SearchToken(val key: String, val value: String, val start: Int, val end: Int)

data class ParsedSearchQuery(val tokens: List<SearchToken>, val text: String) {
    fun values(key: String): List<String> = tokens.filter { it.key == key }.map { it.value }
}

fun parseSearchQuery(query: String): ParsedSearchQuery {
    val tokens = mutableListOf<SearchToken>()
    val text = mutableListOf<String>()
    for (match in Regex("\\S+").findAll(query)) {
        val word = match.value
        val colon = word.indexOf(':')
        val key = if (colon > 0) canonicalSearchKey(word.substring(0, colon)) else null
        if (key != null) {
            val value = word.substring(colon + 1)
            if (value.isNotEmpty()) tokens += SearchToken(key, value, match.range.first, match.range.last + 1)
        } else {
            text += word
        }
    }
    return ParsedSearchQuery(tokens, text.joinToString(" "))
}

fun resolveSearchDate(value: String): String {
    val zone = TimeZone.currentSystemDefault()
    val now = Clock.System.now()
    return when (value.lowercase()) {
        "today" -> now.toLocalDateTime(zone).date.toString()
        "yesterday" -> now.minus(1.days).toLocalDateTime(zone).date.toString()
        else -> value
    }
}

fun Message.matchesSearchExclusion(kind: String): Boolean = when (kind) {
    "embed" -> embeds.isNotEmpty()
    "file" -> attachments.isNotEmpty()
    "image" -> attachments.any { it.content_type?.startsWith("image/") == true }
    "video" -> attachments.any { it.content_type?.startsWith("video/") == true }
    "sound" -> attachments.any { it.content_type?.startsWith("audio/") == true }
    "sticker" -> !sticker_items.isNullOrEmpty() || !stickers.isNullOrEmpty()
    "poll" -> poll != null
    "link" -> Regex("https?://\\S+").containsMatchIn(content)
    else -> false
}

fun withChannelScope(query: String, channelName: String?): String {
    if (channelName.isNullOrBlank()) return query
    if (parseSearchQuery(query).values("in").isNotEmpty()) return query
    return if (query.isBlank()) "in:$channelName " else "in:$channelName $query"
}

private const val DISCORD_EPOCH_MILLIS = 1_420_070_400_000L
private const val DAY_MILLIS = 86_400_000L

private fun searchDateMillis(date: String): Long? {
    val parts = date.split("-")
    if (parts.size != 3) return null
    val year = parts[0].toIntOrNull() ?: return null
    val month = parts[1].toIntOrNull() ?: return null
    val day = parts[2].toIntOrNull() ?: return null
    val y = if (month <= 2) year - 1 else year
    val era = y / 400
    val yoe = y - era * 400
    val mp = (month + 9) % 12
    val doy = (153 * mp + 2) / 5 + day - 1
    val doe = yoe * 365 + yoe / 4 - yoe / 100 + doy
    val epochDay = era.toLong() * 146097L + doe - 719468L
    return epochDay * DAY_MILLIS
}

private fun snowflakeFromMillis(millis: Long): Long = (millis - DISCORD_EPOCH_MILLIS).coerceAtLeast(0L) shl 22

fun searchDateBounds(key: String, date: String): Pair<String?, String?>? {
    val start = searchDateMillis(date) ?: return null
    val nextStart = start + DAY_MILLIS
    return when (key) {
        "before" -> null to snowflakeFromMillis(nextStart).toString()
        "after" -> (snowflakeFromMillis(start) - 1).toString() to null
        else -> (snowflakeFromMillis(start) - 1).toString() to snowflakeFromMillis(nextStart).toString()
    }
}

