package me.lampu.lampcord.shared.state

import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.model.Message
import org.jetbrains.compose.resources.StringResource
import kotlin.time.Duration.Companion.days

private val HAS_CHOICES = listOf("link", "embed", "file", "image", "video", "sound", "sticker", "poll")

enum class SearchFilterKind { USER, CHANNEL, CHOICE, DATE }

data class SearchFilterDef(
    val key: String,
    val label: StringResource,
    val kind: SearchFilterKind,
    val choices: List<String> = emptyList(),
)

val SEARCH_FILTER_DEFS: List<SearchFilterDef> = listOf(
    SearchFilterDef("from", Res.string.search_filter_label_from, SearchFilterKind.USER),
    SearchFilterDef("mentions", Res.string.search_filter_label_mentions, SearchFilterKind.USER),
    SearchFilterDef("in", Res.string.search_filter_label_in, SearchFilterKind.CHANNEL),
    SearchFilterDef("has", Res.string.search_filter_label_has, SearchFilterKind.CHOICE, HAS_CHOICES),
    SearchFilterDef("exclude", Res.string.search_filter_label_exclude, SearchFilterKind.CHOICE, HAS_CHOICES),
    SearchFilterDef("authortype", Res.string.search_filter_label_author_type, SearchFilterKind.CHOICE, listOf("user", "bot", "webhook")),
    SearchFilterDef("before", Res.string.search_filter_label_before, SearchFilterKind.DATE),
    SearchFilterDef("during", Res.string.search_filter_label_during, SearchFilterKind.DATE),
    SearchFilterDef("after", Res.string.search_filter_label_after, SearchFilterKind.DATE),
    SearchFilterDef("sort", Res.string.search_filter_label_sort, SearchFilterKind.CHOICE, listOf("new", "old")),
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
