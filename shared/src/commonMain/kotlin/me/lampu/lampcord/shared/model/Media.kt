package me.lampu.lampcord.shared.model

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
            val type = when (this) {
                is Attachment -> content_type
                is EmbedVideo -> "video/"
                is EmbedImage -> "image/"
                else -> null
            } ?: return MediaKind.OTHER
            
            return when {
                type.startsWith("image/") -> MediaKind.IMAGE
                type.startsWith("video/") -> MediaKind.VIDEO
                type.startsWith("audio/") -> MediaKind.AUDIO
                else -> MediaKind.OTHER
            }
        }

    fun isVideo(): Boolean = mediaKind == MediaKind.VIDEO && !isAudioFilename()
    fun isImage(): Boolean = mediaKind == MediaKind.IMAGE
    fun isAudio(): Boolean = mediaKind == MediaKind.AUDIO || isAudioFilename()

    private fun isAudioFilename(): Boolean =
        (this as? Attachment)?.filename?.substringAfterLast('.', "")?.lowercase() in audioExtensions

    private companion object {
        val audioExtensions = setOf(
            "aac", "flac", "m4a", "mp3", "oga", "ogg", "opus", "wav", "weba"
        )
        val animatedImageExtensions = setOf("gif", "webp", "apng")
        val animatedImageContentTypes = setOf("image/gif", "image/webp", "image/apng")
    }

    fun isGifv(): Boolean {
        val u = (url ?: proxy_url)?.lowercase() ?: return false
        return u.contains("klipy.com") || u.contains(".gifv") || u.contains("tenor.com")
    }

    fun isAnimatedImage(): Boolean {
        if (this is EmbedImage) return true
        val attachment = this as? Attachment ?: return false
        val extension = attachment.filename.substringAfterLast('.', "").lowercase()
        if (extension in animatedImageExtensions) return true
        if (attachment.content_type?.lowercase() in animatedImageContentTypes) return true
        val path = url.substringBefore('?').lowercase()
        return animatedImageExtensions.any { path.endsWith(".$it") }
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
