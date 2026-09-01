package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.patch
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.forms.MultiPartFormDataContent
import io.ktor.client.request.forms.formData
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.Headers
import io.ktor.http.HttpHeaders
import io.ktor.http.contentType
import io.ktor.http.encodeURLQueryComponent
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.Serializable
import me.lampu.lampcord.shared.model.AllowedMentions
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.MessageReference
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.model.SearchResponse
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Logging

@Serializable
data class MessageRequest(
    val content: String,
    val message_reference: MessageReference? = null,
    val tts: Boolean = false,
    val nonce: String? = null,
    val attachments: List<AttachmentRequest>? = null,
    val sticker_ids: List<String>? = null,
    val allowed_mentions: AllowedMentions? = null,
    val poll: Poll? = null
)

@Serializable
data class AttachmentRequest(
    val id: String,
    val filename: String,
    val description: String? = null
)

/**
 * Message lifecycle, reactions and message search endpoints.
 */
class MessageApi(private val rest: RestClient) {

    suspend fun sendMessage(
        channelId: String,
        content: String,
        replyTo: String? = null,
        forwardFrom: Message? = null,
        files: List<Pair<String, ByteArray>> = emptyList(),
        nonce: String? = null,
        stickerIds: List<String>? = null,
        allowedMentions: AllowedMentions? = null,
        poll: Poll? = null
    ): Message? {
        val settings = Settings.shared
        val bypass = settings.bypassUploadLimit

        var finalContent = transformContent(content)
        if (finalContent != content) {
            Logging.i("Message", "Transformed nitro emojis: $content -> $finalContent")
        }

        val remainingFiles = mutableListOf<Pair<String, ByteArray>>()
        if (bypass) {
            files.forEach { (name, bytes) ->
                if (bytes.size > 10 * 1024 * 1024) {
                    val url = uploadToCatbox(name, bytes)
                    if (url != null) {
                        finalContent += (if (finalContent.isNotEmpty()) "\n" else "") + url
                    } else {
                        remainingFiles.add(name to bytes)
                    }
                } else {
                    remainingFiles.add(name to bytes)
                }
            }
        } else {
            remainingFiles.addAll(files)
        }

        val messageReference = when {
            replyTo != null -> MessageReference(message_id = replyTo)
            forwardFrom != null -> MessageReference(
                type = 1,
                message_id = forwardFrom.id,
                channel_id = forwardFrom.channel_id,
                guild_id = forwardFrom.guild_id
            )
            else -> null
        }

        return try {
            if (remainingFiles.isEmpty()) {
                val response = rest.httpClient.post("${rest.apiBase}/channels/$channelId/messages") {
                    standardHeaders(rest)
                    contentType(ContentType.Application.Json)

                    val request = MessageRequest(
                        content = finalContent,
                        message_reference = messageReference,
                        nonce = nonce,
                        sticker_ids = stickerIds,
                        allowed_mentions = allowedMentions,
                        poll = poll
                    )
                    setBody(request)
                }
                if (response.status.isSuccess()) response.body() else null
            } else {
                val response = rest.httpClient.post("${rest.apiBase}/channels/$channelId/messages") {
                    standardHeaders(rest)

                    val attachmentMetadata = remainingFiles.mapIndexed { index, (name, _) ->
                        AttachmentRequest(id = index.toString(), filename = name)
                    }

                    setBody(MultiPartFormDataContent(
                        formData {
                            append("payload_json", rest.json.encodeToString(MessageRequest(
                                content = finalContent,
                                message_reference = messageReference,
                                nonce = nonce,
                                attachments = attachmentMetadata,
                                sticker_ids = stickerIds,
                                allowed_mentions = allowedMentions,
                                poll = poll
                            )), Headers.build {
                                append(HttpHeaders.ContentType, ContentType.Application.Json.toString())
                            })
                            remainingFiles.forEachIndexed { index, (name, bytes) ->
                                append("files[$index]", bytes, Headers.build {
                                    append(HttpHeaders.ContentType, getContentTypeForFile(name).toString())
                                    append(HttpHeaders.ContentDisposition, "form-data; name=\"files[$index]\"; filename=\"$name\"")
                                })
                            }
                        }
                    ))
                }
                if (response.status.isSuccess()) response.body() else null
            }
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Messages", "Error sending message: ${e.message}")
            null
        }
    }

    suspend fun uploadToCatbox(name: String, bytes: ByteArray): String? {
        return try {
            val response = rest.httpClient.post("https://catbox.moe/user/api.php") {
                setBody(MultiPartFormDataContent(
                    formData {
                        append("reqtype", "fileupload")
                        append("fileToUpload", bytes, Headers.build {
                            append(HttpHeaders.ContentType, getContentTypeForFile(name).toString())
                            append(HttpHeaders.ContentDisposition, "form-data; name=\"fileToUpload\"; filename=\"$name\"")
                        })
                    }
                ))
            }
            val text = response.bodyAsText()
            if (text.startsWith("https://")) text.trim() else null
        } catch (e: Exception) {
            Logging.e("Upload", "Catbox upload failed: ${e.message}")
            null
        }
    }

    private fun getContentTypeForFile(name: String): ContentType {
        return when {
            name.endsWith(".png", true) -> ContentType.Image.PNG
            name.endsWith(".jpg", true) || name.endsWith(".jpeg", true) -> ContentType.Image.JPEG
            name.endsWith(".gif", true) -> ContentType.Image.GIF
            name.endsWith(".webp", true) -> ContentType.parse("image/webp")
            name.endsWith(".mp4", true) -> ContentType.Video.MP4
            name.endsWith(".mov", true) -> ContentType.Video.QuickTime
            name.endsWith(".webm", true) -> ContentType.Video.Any
            name.endsWith(".m4a", true) -> ContentType.parse("audio/mp4")
            name.endsWith(".aac", true) -> ContentType.parse("audio/aac")
            name.endsWith(".mp3", true) -> ContentType.parse("audio/mpeg")
            name.endsWith(".ogg", true) || name.endsWith(".opus", true) -> ContentType.parse("audio/ogg")
            name.endsWith(".wav", true) -> ContentType.parse("audio/wav")
            name.endsWith(".3gp", true) -> ContentType.parse("audio/3gpp")
            else -> ContentType.Application.OctetStream
        }
    }

    suspend fun getChannelMessages(channelId: String, limit: Int = 50, before: String? = null): List<Message> {
        return getChannelMessagesPage(channelId, limit, before) ?: emptyList()
    }

    /**
     * Returns null when the request failed, allowing history pagination to
     * distinguish a retryable failure from a successful end-of-history page.
     */
    suspend fun getChannelMessagesPage(channelId: String, limit: Int = 50, before: String? = null): List<Message>? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/channels/$channelId/messages") {
                standardHeaders(rest)
                parameter("limit", limit)
                if (before != null) {
                    parameter("before", before)
                }
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Messages", "Error fetching messages: ${e.message}")
            null
        }
    }

    suspend fun editMessage(channelId: String, messageId: String, content: String): Boolean {
        return try {
            val response = rest.httpClient.patch("${rest.apiBase}/channels/$channelId/messages/$messageId") {
                standardHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(MessageRequest(transformContent(content)))
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Message", "Error editing message: ${e.message}")
            false
        }
    }

    private fun transformContent(content: String): String {
        val settings = Settings.shared
        if (!settings.freeNitroEmojis) return content

        val emojiRegex = Regex("""<(a?):F_([a-zA-Z0-9_]+):(\d+)>""")
        return content.replace(emojiRegex) { match ->
            val animated = match.groupValues[1] == "a"
            val name = match.groupValues[2]
            val id = match.groupValues[3]
            val useWebp = settings.useWebpEmojis

            val url = if (useWebp) {
                "https://cdn.discordapp.com/emojis/$id.webp?name=$name&animated=$animated&size=48"
            } else {
                val ext = if (animated) "gif" else "png"
                "https://cdn.discordapp.com/emojis/$id.$ext?name=$name&size=48"
            }

            if (settings.realmojis) "[$name]($url)" else url
        }
    }

    suspend fun deleteMessage(channelId: String, messageId: String): Boolean {
        return try {
            val response = rest.httpClient.delete("${rest.apiBase}/channels/$channelId/messages/$messageId") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Message", "Error deleting message: ${e.message}")
            false
        }
    }

    suspend fun addReaction(channelId: String, messageId: String, emoji: String): Boolean {
        return try {
            val encodedEmoji = emoji.encodeURLQueryComponent()
            val response = rest.httpClient.put("${rest.apiBase}/channels/$channelId/messages/$messageId/reactions/$encodedEmoji/@me") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Reaction", "Error adding reaction: ${e.message}")
            false
        }
    }

    suspend fun removeReaction(channelId: String, messageId: String, emoji: String): Boolean {
        return try {
            val encodedEmoji = emoji.encodeURLQueryComponent()
            val response = rest.httpClient.delete("${rest.apiBase}/channels/$channelId/messages/$messageId/reactions/$encodedEmoji/@me") {
                standardHeaders(rest)
            }
            response.status.isSuccess()
        } catch (e: Exception) {
            Logging.e("Reaction", "Error removing reaction: ${e.message}")
            false
        }
    }

    suspend fun getReactionUsers(channelId: String, messageId: String, emoji: String): List<me.lampu.lampcord.shared.model.User> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/channels/$channelId/messages/$messageId/reactions/$emoji") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Reaction", "Error fetching reaction users: ${e.message}")
            emptyList()
        }
    }

    suspend fun searchChannelMessages(
        channelId: String,
        content: String? = null,
        authorId: String? = null,
        mentions: String? = null,
        has: String? = null,
        before: String? = null,
        after: String? = null,
        during: String? = null,
        sort: String? = null,
        authorType: String? = null
    ): SearchResponse? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/channels/$channelId/messages/search") {
                standardHeaders(rest)
                content?.let { parameter("content", it) }
                authorId?.let { parameter("author_id", it) }
                mentions?.let { parameter("mentions", it) }
                has?.let { parameter("has", it) }
                before?.let { parameter("before", it) }
                after?.let { parameter("after", it) }
                during?.let { parameter("during", it) }
                sort?.let { parameter("sort_by", it) }
                authorType?.let { parameter("author_type", it) }
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Messages", "Error searching channel messages: ${e.message}")
            null
        }
    }

    suspend fun searchGuildMessages(
        guildId: String,
        content: String? = null,
        authorId: String? = null,
        mentions: String? = null,
        has: String? = null,
        channelId: String? = null,
        before: String? = null,
        after: String? = null,
        during: String? = null,
        sort: String? = null,
        authorType: String? = null
    ): SearchResponse? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/guilds/$guildId/messages/search") {
                standardHeaders(rest)
                content?.let { parameter("content", it) }
                authorId?.let { parameter("author_id", it) }
                mentions?.let { parameter("mentions", it) }
                has?.let { parameter("has", it) }
                channelId?.let { parameter("channel_id", it) }
                before?.let { parameter("before", it) }
                after?.let { parameter("after", it) }
                during?.let { parameter("during", it) }
                sort?.let { parameter("sort_by", it) }
                authorType?.let { parameter("author_type", it) }
            }
            if (response.status.isSuccess()) response.body() else null
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Logging.e("Messages", "Error searching guild messages: ${e.message}")
            null
        }
    }

    suspend fun getMentions(limit: Int = 20, before: String? = null): List<Message> {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/users/@me/mentions") {
                standardHeaders(rest)
                parameter("limit", limit)
                if (before != null) {
                    parameter("before", before)
                }
            }
            if (response.status.isSuccess()) response.body() else emptyList()
        } catch (e: Exception) {
            Logging.e("Messages", "Error fetching mentions: ${e.message}")
            emptyList()
        }
    }
}
