package me.lampu.lampcord.shared.image

import com.github.panpf.sketch.decode.Decoder

private val PNG_SIGNATURE = byteArrayOf(0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)

// The [Decoder.Factory] for APNG images on the current platform (Android or desktop JVM). iOS lacks an APNG decoder and renders decorations as their static preview instead.
expect fun apngDecoderFactory(): Decoder.Factory

data class ApngFrameMeta(
    val x: Int,
    val y: Int,
    val width: Int,
    val height: Int,
    val delayNum: Int,
    val delayDen: Int,
    val disposeOp: Int,
    val blendOp: Int,
)

data class ApngInfo(
    val width: Int,
    val height: Int,
    val numFrames: Int,
    val numPlays: Int,
    val frames: List<ApngFrameMeta>,
)

// Detect APNG: valid PNG signature plus an `acTL` chunk appearing before the first `IDAT`.
fun isApngFile(headerBytes: ByteArray): Boolean {
    if (headerBytes.size < 8) return false
    for (i in 0 until PNG_SIGNATURE.size) {
        if (headerBytes[i] != PNG_SIGNATURE[i]) return false
    }
    var offset = 8
    while (offset + 8 <= headerBytes.size) {
        val length = readIntBE(headerBytes, offset)
        if (length < 0) return false
        val type = String(headerBytes, offset + 4, 4, Charsets.US_ASCII)
        when (type) {
            "acTL" -> return true
            "IDAT" -> return false
        }
        offset += 12 + length
    }
    return false
}

// Parse the APNG header chunks (`IHDR`, `acTL`, `fcTL`) from the full file.
fun parseApngInfo(bytes: ByteArray): ApngInfo? {
    if (!isApngFile(bytes)) return null
    var offset = 8
    var width = 0
    var height = 0
    var numFrames = 0
    var numPlays = 0
    val frames = ArrayList<ApngFrameMeta>()
    while (offset + 12 <= bytes.size) {
        val length = readIntBE(bytes, offset)
        if (length < 0 || offset + 8 + length > bytes.size) break
        val type = String(bytes, offset + 4, 4, Charsets.US_ASCII)
        val data = offset + 8
        when (type) {
            "IHDR" -> {
                if (length < 8) return null
                width = readIntBE(bytes, data)
                height = readIntBE(bytes, data + 4)
            }
            "acTL" -> {
                if (length < 8) return null
                numFrames = readIntBE(bytes, data)
                numPlays = readIntBE(bytes, data + 4)
            }
            "fcTL" -> {
                if (length < 25) return null
                val w = readIntBE(bytes, data + 4)
                val h = readIntBE(bytes, data + 8)
                val x = readIntBE(bytes, data + 12)
                val y = readIntBE(bytes, data + 16)
                val delayNum = readUShortBE(bytes, data + 20)
                val delayDen = readUShortBE(bytes, data + 22)
                val disposeOp = bytes[data + 24].toInt() and 0xFF
                val blendOp = bytes[data + 25].toInt() and 0xFF
                frames.add(
                    ApngFrameMeta(
                        x = x, y = y, width = w, height = h,
                        delayNum = delayNum, delayDen = delayDen,
                        disposeOp = disposeOp, blendOp = blendOp,
                    )
                )
            }
            "IEND" -> break
        }
        offset += 12 + length
    }
    if (width == 0 || height == 0 || numFrames == 0) return null
    return ApngInfo(width, height, numFrames, numPlays, frames)
}

private fun readIntBE(bytes: ByteArray, offset: Int): Int {
    return ((bytes[offset].toInt() and 0xFF) shl 24) or
            ((bytes[offset + 1].toInt() and 0xFF) shl 16) or
            ((bytes[offset + 2].toInt() and 0xFF) shl 8) or
            (bytes[offset + 3].toInt() and 0xFF)
}

private fun readUShortBE(bytes: ByteArray, offset: Int): Int {
    return ((bytes[offset].toInt() and 0xFF) shl 8) or
            (bytes[offset + 1].toInt() and 0xFF)
}