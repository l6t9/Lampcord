package me.lampu.lampcord.shared.utils

/**
 * ThumbHash is a compact image placeholder format.
 * This is a minimal implementation to decode ThumbHash strings.
 */
object ThumbHash {
    fun thumbHashToRGBA(hash: ByteArray): RGBA {
        // A minimal implementation that extracts average colors from the header.
        // In a full implementation, we would decode the DCT coefficients.
        if (hash.size < 4) return RGBA(0f, 0f, 0f, 0f)

        // Very simplified: use the first 3 bytes as RGB
        val r = (hash[0].toInt() and 255).toFloat() / 255f
        val g = (hash[1].toInt() and 255).toFloat() / 255f
        val b = (hash[2].toInt() and 255).toFloat() / 255f
        
        return RGBA(r, g, b, 1f)
    }

    data class RGBA(val r: Float, val g: Float, val b: Float, val a: Float)
}
