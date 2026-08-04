package com.example.lampcord.shared.model

import kotlinx.serialization.Serializable

interface DiscordMedia {
    val url: String?
    val proxy_url: String?
    val height: Int?
    val width: Int?
    val placeholder: String?
    
    val aspectRatio: Float?
        get() {
            val h = height?.toFloat() ?: return null
            val w = width?.toFloat() ?: return null
            if (h == 0f) return null
            return w / h
        }

    val mediaKind: MediaKind
        get() {
            val type = (this as? Attachment)?.content_type ?: return MediaKind.OTHER
            return when {
                type.startsWith("image/") -> MediaKind.IMAGE
                type.startsWith("video/") -> MediaKind.VIDEO
                type.startsWith("audio/") -> MediaKind.AUDIO
                else -> MediaKind.OTHER
            }
        }
}

enum class MediaKind {
    IMAGE, VIDEO, AUDIO, OTHER
}

@Serializable
data class Attachment(
    val id: String,
    val filename: String,
    val description: String? = null,
    val content_type: String? = null,
    val size: Int,
    override val url: String,
    override val proxy_url: String,
    override val height: Int? = null,
    override val width: Int? = null,
    override val placeholder: String? = null,
    val ephemeral: Boolean? = null,
    val flags: Int? = null
) : DiscordMedia

@Serializable
data class Embed(
    val title: String? = null,
    val type: String? = null,
    val description: String? = null,
    val url: String? = null,
    val timestamp: String? = null,
    val color: Int? = null,
    val footer: EmbedFooter? = null,
    val image: EmbedImage? = null,
    val thumbnail: EmbedImage? = null,
    val video: EmbedVideo? = null,
    val provider: EmbedProvider? = null,
    val author: EmbedAuthor? = null,
    val fields: List<EmbedField>? = null
)

@Serializable
data class EmbedFooter(val text: String, val icon_url: String? = null, val proxy_icon_url: String? = null)

@Serializable
data class EmbedImage(
    override val url: String? = null,
    override val proxy_url: String? = null,
    override val height: Int? = null,
    override val width: Int? = null,
    override val placeholder: String? = null
) : DiscordMedia

@Serializable
data class EmbedVideo(
    override val url: String? = null,
    override val proxy_url: String? = null,
    override val height: Int? = null,
    override val width: Int? = null,
    override val placeholder: String? = null
) : DiscordMedia

@Serializable
data class EmbedProvider(val name: String? = null, val url: String? = null)

@Serializable
data class EmbedAuthor(val name: String, val url: String? = null, val icon_url: String? = null, val proxy_icon_url: String? = null)

@Serializable
data class EmbedField(val name: String, val value: String, val inline: Boolean = false)
