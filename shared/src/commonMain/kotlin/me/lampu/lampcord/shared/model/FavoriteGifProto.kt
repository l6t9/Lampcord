package me.lampu.lampcord.shared.model

import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

/**
 * Minimal decoder for Discord's FrecencyUserSettings favorite_gifs field.
 * Keeping this small avoids making the whole protobuf schema a runtime
 * dependency just to read the picker favorites.
 */
@OptIn(ExperimentalEncodingApi::class)
fun decodeFavoriteGifs(base64: String): List<Gif> {
    return try {
        val reader = ProtoReader(Base64.Default.decode(base64))
        val favorites = mutableListOf<OrderedFavoriteGif>()

        while (reader.hasRemaining()) {
            val tag = reader.readTag() ?: break
            if (tag.fieldNumber == 2 && tag.wireType == 2) {
                parseFavoriteGifs(reader.readBytes(), favorites)
            } else {
                reader.skip(tag.wireType)
            }
        }

        favorites
            .sortedWith(compareBy<OrderedFavoriteGif> { it.order }.thenBy { it.url })
            .map { favorite ->
                Gif(
                    src = favorite.src,
                    url = favorite.url,
                    width = favorite.width,
                    height = favorite.height,
                    preview = favorite.preview,
                    isVideo = favorite.isVideo
                )
            }
    } catch (_: Exception) {
        emptyList()
    }
}

private data class OrderedFavoriteGif(
    val url: String,
    val src: String,
    val width: Int,
    val height: Int,
    val order: Int,
    val isVideo: Boolean,
    val preview: String? = null
)

private fun parseFavoriteGifs(bytes: ByteArray, output: MutableList<OrderedFavoriteGif>) {
    val reader = ProtoReader(bytes)
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        if (tag.fieldNumber == 1 && tag.wireType == 2) {
            parseGifMapEntry(reader.readBytes(), output)
        } else {
            reader.skip(tag.wireType)
        }
    }
}

private fun parseGifMapEntry(bytes: ByteArray, output: MutableList<OrderedFavoriteGif>) {
    val reader = ProtoReader(bytes)
    var url = ""
    var src = ""
    var width = 0
    var height = 0
    var order = 0
    var isVideo = false

    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        when (tag.fieldNumber) {
            1 -> if (tag.wireType == 2) url = reader.readString()
            else reader.skip(tag.wireType)
            2 -> if (tag.wireType == 2) {
                val gif = parseFavoriteGif(reader.readBytes())
                src = gif.src
                width = gif.width
                height = gif.height
                order = gif.order
                isVideo = gif.isVideo
            } else reader.skip(tag.wireType)
            else -> reader.skip(tag.wireType)
        }
    }

    if (url.isNotBlank() && src.isNotBlank()) {
        output += OrderedFavoriteGif(url, src, width, height, order, isVideo)
    }
}

private data class ParsedFavoriteGif(
    val src: String = "",
    val width: Int = 0,
    val height: Int = 0,
    val order: Int = 0,
    val isVideo: Boolean = false
)

private fun parseFavoriteGif(bytes: ByteArray): ParsedFavoriteGif {
    val reader = ProtoReader(bytes)
    var result = ParsedFavoriteGif()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        when (tag.fieldNumber) {
            1, 3, 4, 5 -> if (tag.wireType == 0) {
                val value = reader.readVarint().toInt()
                result = when (tag.fieldNumber) {
                    3 -> result.copy(width = value)
                    4 -> result.copy(height = value)
                    5 -> result.copy(order = value)
                    1 -> result.copy(isVideo = value == 2)
                    else -> result
                }
            } else reader.skip(tag.wireType)
            2 -> if (tag.wireType == 2) result = result.copy(src = reader.readString())
            else reader.skip(tag.wireType)
            else -> reader.skip(tag.wireType)
        }
    }
    return result
}

private data class ProtoTag(val fieldNumber: Int, val wireType: Int)

private class ProtoReader(private val bytes: ByteArray) {
    private var position = 0

    fun hasRemaining(): Boolean = position < bytes.size

    fun readTag(): ProtoTag? {
        if (!hasRemaining()) return null
        val tag = readVarint().toInt()
        return ProtoTag(tag ushr 3, tag and 7)
    }

    fun readVarint(): Long {
        var result = 0L
        var shift = 0
        while (position < bytes.size && shift < 64) {
            val byte = bytes[position++].toInt() and 0xFF
            result = result or ((byte and 0x7F).toLong() shl shift)
            if ((byte and 0x80) == 0) return result
            shift += 7
        }
        throw IllegalArgumentException("Invalid protobuf varint")
    }

    fun readBytes(): ByteArray {
        val length = readVarint().toInt()
        if (length < 0 || position + length > bytes.size) {
            throw IllegalArgumentException("Invalid protobuf length")
        }
        return bytes.copyOfRange(position, position + length).also { position += length }
    }

    fun readString(): String = readBytes().decodeToString()

    fun skip(wireType: Int) {
        when (wireType) {
            0 -> readVarint()
            1 -> skipBytes(8)
            2 -> skipBytes(readVarint().toInt())
            5 -> skipBytes(4)
            else -> throw IllegalArgumentException("Unsupported protobuf wire type")
        }
    }

    private fun skipBytes(length: Int) {
        if (length < 0 || position + length > bytes.size) {
            throw IllegalArgumentException("Invalid protobuf field length")
        }
        position += length
    }
}
