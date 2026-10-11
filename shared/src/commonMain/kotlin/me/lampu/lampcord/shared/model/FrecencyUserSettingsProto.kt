package me.lampu.lampcord.shared.model

import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

data class FrecencyUserSettings(
    val favoriteEmojiKeys: List<String> = emptyList(),
    val favoriteStickerIds: List<String> = emptyList(),
    val emojiFrecency: Map<String, FrecencyItem> = emptyMap(),
    val stickerFrecency: Map<String, FrecencyItem> = emptyMap()
)

data class FrecencyItem(
    val totalUses: Int = 0,
    val recentUses: List<Long> = emptyList(),
    val frecency: Int = -1,
    val score: Int = 0
)

class ProtoTag2(val fieldNumber: Int, val wireType: Int)

class ProtoReader2(val bytes: ByteArray) {
    var position = 0

    fun hasRemaining(): Boolean = position < bytes.size

    fun readTag(): ProtoTag2? {
        if (!hasRemaining()) return null
        val tag = readVarint().toInt()
        return ProtoTag2(tag ushr 3, tag and 7)
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
            throw IllegalArgumentException("Invalid protobuf field length")
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

@OptIn(ExperimentalEncodingApi::class)
fun decodeFrecencyUserSettings(base64: String): FrecencyUserSettings {
    return try {
        val bytes = Base64.Default.decode(base64)
        val reader = ProtoReader2(bytes)
        var favoriteEmojis = listOf<String>()
        var favoriteStickers = listOf<String>()
        var emojiFrecency = mapOf<String, FrecencyItem>()
        var stickerFrecency = mapOf<String, FrecencyItem>()

        while (reader.hasRemaining()) {
            val tag = reader.readTag() ?: break
            when (tag.fieldNumber) {
                6 -> if (tag.wireType == 2) favoriteEmojis = parseFavoriteEmojis(reader.readBytes()).emojis
                else reader.skip(tag.wireType)
                4 -> if (tag.wireType == 2) favoriteStickers = parseFavoriteStickers(reader.readBytes()).stickerIds.map { it.toString() }
                else reader.skip(tag.wireType)
                7 -> if (tag.wireType == 2) emojiFrecency = parseStringFrecencyMap(reader.readBytes())
                else reader.skip(tag.wireType)
                5 -> if (tag.wireType == 2) stickerFrecency = parseFixed64FrecencyMap(reader.readBytes())
                else reader.skip(tag.wireType)
                else -> reader.skip(tag.wireType)
            }
        }

        FrecencyUserSettings(
            favoriteEmojiKeys = favoriteEmojis,
            favoriteStickerIds = favoriteStickers,
            emojiFrecency = emojiFrecency,
            stickerFrecency = stickerFrecency
        )
    } catch (_: Exception) {
        FrecencyUserSettings()
    }
}

private fun parseFavoriteEmojis(bytes: ByteArray): FavoriteEmojis {
    val reader = ProtoReader2(bytes)
    val emojis = mutableListOf<String>()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        if (tag.fieldNumber == 1 && tag.wireType == 2) {
            emojis += reader.readString()
        } else {
            reader.skip(tag.wireType)
        }
    }
    return FavoriteEmojis(emojis)
}

private fun parseFavoriteStickers(bytes: ByteArray): FavoriteStickers {
    val reader = ProtoReader2(bytes)
    val ids = mutableListOf<Long>()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        if (tag.fieldNumber == 1 && tag.wireType == 1) {
            val start = reader.position
            if (start + 8 <= bytes.size) {
                var value = 0L
                for (i in 7 downTo 0) {
                    value = (value shl 8) or (bytes[start + i].toLong() and 0xFF)
                }
                ids += value
                reader.position = start + 8
            }
        } else {
            reader.skip(tag.wireType)
        }
    }
    return FavoriteStickers(ids)
}

private fun parseStringFrecencyMap(bytes: ByteArray): Map<String, FrecencyItem> {
    val reader = ProtoReader2(bytes)
    val map = mutableMapOf<String, FrecencyItem>()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        if (tag.fieldNumber == 1 && tag.wireType == 2) {
            parseStringFrecencyEntry(reader.readBytes(), map)
        } else {
            reader.skip(tag.wireType)
        }
    }
    return map
}

private fun parseStringFrecencyEntry(bytes: ByteArray, map: MutableMap<String, FrecencyItem>) {
    val reader = ProtoReader2(bytes)
    var key = ""
    var item = FrecencyItem()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        when (tag.fieldNumber) {
            1 -> if (tag.wireType == 2) key = reader.readString()
            else reader.skip(tag.wireType)
            2 -> if (tag.wireType == 2) item = parseFrecencyItem(reader.readBytes())
            else reader.skip(tag.wireType)
            else -> reader.skip(tag.wireType)
        }
    }
    if (key.isNotBlank()) map[key] = item
}

private fun parseFixed64FrecencyMap(bytes: ByteArray): Map<String, FrecencyItem> {
    val reader = ProtoReader2(bytes)
    val map = mutableMapOf<String, FrecencyItem>()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        if (tag.fieldNumber == 1 && tag.wireType == 2) {
            parseFixed64FrecencyEntry(reader.readBytes(), map)
        } else {
            reader.skip(tag.wireType)
        }
    }
    return map
}

private fun parseFixed64FrecencyEntry(bytes: ByteArray, map: MutableMap<String, FrecencyItem>) {
    val reader = ProtoReader2(bytes)
    var key: Long = 0
    var item = FrecencyItem()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        when (tag.fieldNumber) {
            1 -> if (tag.wireType == 1) {
                val start = reader.position
                if (start + 8 <= bytes.size) {
                    var value = 0L
                    for (i in 7 downTo 0) {
                        value = (value shl 8) or (bytes[start + i].toLong() and 0xFF)
                    }
                    key = value
                    reader.position = start + 8
                }
            } else reader.skip(tag.wireType)
            2 -> if (tag.wireType == 2) item = parseFrecencyItem(reader.readBytes())
            else reader.skip(tag.wireType)
            else -> reader.skip(tag.wireType)
        }
    }
    if (key != 0L) map[key.toString()] = item
}

private fun parseFrecencyItem(bytes: ByteArray): FrecencyItem {
    val reader = ProtoReader2(bytes)
    var totalUses = 0
    val recentUses = mutableListOf<Long>()
    var frecency = -1
    var score = 0
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        when (tag.fieldNumber) {
            1 -> if (tag.wireType == 0) totalUses = reader.readVarint().toInt()
            else reader.skip(tag.wireType)
            2 -> if (tag.wireType == 2) recentUses += reader.readBytes().let { parseVarintList(it) }
            else reader.skip(tag.wireType)
            3 -> if (tag.wireType == 0) frecency = reader.readVarint().toInt()
            else reader.skip(tag.wireType)
            4 -> if (tag.wireType == 0) score = reader.readVarint().toInt()
            else reader.skip(tag.wireType)
            else -> reader.skip(tag.wireType)
        }
    }
    return FrecencyItem(totalUses, recentUses, frecency, score)
}

private fun parseVarintList(bytes: ByteArray): List<Long> {
    val reader = ProtoReader2(bytes)
    val list = mutableListOf<Long>()
    while (reader.hasRemaining()) {
        val tag = reader.readTag() ?: break
        if (tag.wireType == 0) {
            list += reader.readVarint()
        } else {
            reader.skip(tag.wireType)
        }
    }
    return list
}

private data class FavoriteEmojis(val emojis: List<String>)
private data class FavoriteStickers(val stickerIds: List<Long>)

@OptIn(ExperimentalEncodingApi::class)
fun encodeFrecencyUserSettings(
    favoriteEmojis: List<String>,
    favoriteStickers: List<String>,
    emojiFrecency: Map<String, FrecencyItem> = emptyMap(),
    stickerFrecency: Map<String, FrecencyItem> = emptyMap()
): String {
    val out = mutableListOf<Byte>()

    if (favoriteStickers.isNotEmpty()) {
        val payload = buildFavoriteStickersPayload(favoriteStickers.mapNotNull { it.toLongOrNull() })
        writeBytesField(out, 4, payload)
    }
    if (stickerFrecency.isNotEmpty()) {
        val payload = buildFrecencyMapPayload(stickerFrecency, keyIsFixed64 = true)
        writeBytesField(out, 5, payload)
    }
    if (favoriteEmojis.isNotEmpty()) {
        val payload = buildFavoriteEmojisPayload(favoriteEmojis)
        writeBytesField(out, 6, payload)
    }
    if (emojiFrecency.isNotEmpty()) {
        val payload = buildFrecencyMapPayload(emojiFrecency, keyIsFixed64 = false)
        writeBytesField(out, 7, payload)
    }

    return Base64.Default.encode(out.toByteArray())
}

private fun buildFavoriteStickersPayload(ids: List<Long>): ByteArray {
    val out = mutableListOf<Byte>()
    for (id in ids) {
        writeTag(out, 1, 1)
        for (i in 0 until 8) {
            out += ((id ushr (8 * i)) and 0xFF).toByte()
        }
    }
    return out.toByteArray()
}

private fun buildFavoriteEmojisPayload(emojis: List<String>): ByteArray {
    val out = mutableListOf<Byte>()
    for (emoji in emojis) {
        val bytes = emoji.encodeToByteArray()
        writeTag(out, 1, 2)
        writeVarint(out, bytes.size.toLong())
        out += bytes.toList()
    }
    return out.toByteArray()
}

private fun buildFrecencyMapPayload(map: Map<String, FrecencyItem>, keyIsFixed64: Boolean): ByteArray {
    val out = mutableListOf<Byte>()
    for ((key, item) in map) {
        val itemPayload = buildFrecencyItemPayload(item)
        val keyBytes = if (keyIsFixed64) {
            val id = key.toLongOrNull() ?: 0L
            val arr = mutableListOf<Byte>()
            for (i in 0 until 8) {
                arr += ((id ushr (8 * i)) and 0xFF).toByte()
            }
            arr.toByteArray()
        } else {
            key.encodeToByteArray()
        }
        val entry = mutableListOf<Byte>()
        if (keyIsFixed64) {
            writeTag(entry, 1, 1)
            entry += keyBytes.toList()
        } else {
            writeTag(entry, 1, 2)
            writeVarint(entry, keyBytes.size.toLong())
            entry += keyBytes.toList()
        }
        writeTag(entry, 2, 2)
        writeVarint(entry, itemPayload.size.toLong())
        entry += itemPayload.toList()
        writeTag(out, 1, 2)
        writeVarint(out, entry.size.toLong())
        out += entry
    }
    return out.toByteArray()
}

private fun buildFrecencyItemPayload(item: FrecencyItem): ByteArray {
    val out = mutableListOf<Byte>()
    if (item.totalUses != 0) {
        writeTag(out, 1, 0)
        writeVarint(out, item.totalUses.toLong())
    }
    if (item.recentUses.isNotEmpty()) {
        val packed = mutableListOf<Byte>()
        for (v in item.recentUses) {
            writeVarint(packed, v)
        }
        writeTag(out, 2, 2)
        writeVarint(out, packed.size.toLong())
        out += packed.toList()
    }
    if (item.frecency != -1) {
        writeTag(out, 3, 0)
        writeVarint(out, item.frecency.toLong())
    }
    if (item.score != 0) {
        writeTag(out, 4, 0)
        writeVarint(out, item.score.toLong())
    }
    return out.toByteArray()
}

private fun writeTag(out: MutableList<Byte>, fieldNumber: Int, wireType: Int) {
    writeVarint(out, ((fieldNumber shl 3) or wireType).toLong())
}

private fun writeVarint(out: MutableList<Byte>, value: Long) {
    var v = value
    while (v and 0x7FL.inv() != 0L) {
        out += (((v and 0x7F) or 0x80).toByte())
        v = v ushr 7
    }
    out += (v and 0x7F).toByte()
}

private fun writeBytesField(out: MutableList<Byte>, fieldNumber: Int, payload: ByteArray) {
    writeTag(out, fieldNumber, 2)
    writeVarint(out, payload.size.toLong())
    out += payload.toList()
}