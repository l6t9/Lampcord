package me.lampu.lampcord.shared.utils

object ThumbHash {
    fun thumbHashToRGBA(hash: ByteArray): RGBA {
        if (hash.size < 4) return RGBA(0f, 0f, 0f, 0f)

        val r = (hash[0].toInt() and 255).toFloat() / 255f
        val g = (hash[1].toInt() and 255).toFloat() / 255f
        val b = (hash[2].toInt() and 255).toFloat() / 255f
        
        return RGBA(r, g, b, 1f)
    }

    data class RGBA(val r: Float, val g: Float, val b: Float, val a: Float)
}
