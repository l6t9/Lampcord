package me.lampu.lampcord.shared.model

// Discord's VIEW_CHANNEL permission bit (a.k.a. read_messages).
private const val VIEW_CHANNEL = 1L shl 10

// Port of murmurhash32.
fun murmurhash32(key: String, seed: Int = 0): Long {
    val keyBytes = key.encodeToByteArray()
    val length = keyBytes.size
    val nblocks = length / 4

    var h1 = (seed.toLong() and 0xFFFFFFFFL)
    val c1 = 0xCC9E2D51L
    val c2 = 0x1B873593L

    for (blockStart in 0 until nblocks) {
        val i = blockStart * 4
        var k1 = 0L
        k1 = k1 or (keyBytes[i].toLong() and 0xFF)
        k1 = k1 or ((keyBytes[i + 1].toLong() and 0xFF) shl 8)
        k1 = k1 or ((keyBytes[i + 2].toLong() and 0xFF) shl 16)
        k1 = k1 or ((keyBytes[i + 3].toLong() and 0xFF) shl 24)

        k1 = (k1 * c1) and 0xFFFFFFFFL
        k1 = ((k1 shl 15) or (k1 ushr 17)) and 0xFFFFFFFFL
        k1 = (k1 * c2) and 0xFFFFFFFFL

        h1 = h1 xor k1
        h1 = ((h1 shl 13) or (h1 ushr 19)) and 0xFFFFFFFFL
        h1 = (h1 * 5 + 0xE6546B64L) and 0xFFFFFFFFL
    }

    val tailIndex = nblocks * 4
    var k1 = 0L
    val tailSize = length and 3

    if (tailSize >= 3) k1 = k1 xor ((keyBytes[tailIndex + 2].toLong() and 0xFF) shl 16)
    if (tailSize >= 2) k1 = k1 xor ((keyBytes[tailIndex + 1].toLong() and 0xFF) shl 8)
    if (tailSize >= 1) k1 = k1 xor (keyBytes[tailIndex].toLong() and 0xFF)
    if (tailSize > 0) {
        k1 = (k1 * c1) and 0xFFFFFFFFL
        k1 = ((k1 shl 15) or (k1 ushr 17)) and 0xFFFFFFFFL
        k1 = (k1 * c2) and 0xFFFFFFFFL
        h1 = h1 xor k1
    }

    var unsignedVal = h1 xor length.toLong()
    unsignedVal = unsignedVal xor (unsignedVal ushr 16)
    unsignedVal = (unsignedVal * 0x85EBCA6BL) and 0xFFFFFFFFL
    unsignedVal = unsignedVal xor (unsignedVal ushr 13)
    unsignedVal = (unsignedVal * 0xC2B2AE35L) and 0xFFFFFFFFL
    unsignedVal = unsignedVal xor (unsignedVal ushr 16)
    return unsignedVal
}

private fun hasPermission(bits: String?, flag: Long): Boolean {
    val value = bits?.toLongOrNull() ?: return false
    return (value and flag) == flag
}

fun Channel.memberListId(guild: Guild): String {
    val everyoneRole = guild.roles.firstOrNull { it.id == guild.id }
    val everyoneCanView = everyoneRole != null && hasPermission(everyoneRole.permissions, VIEW_CHANNEL)

    val overwrites = permission_overwrites ?: emptyList()

    if (everyoneCanView && overwrites.none { hasPermission(it.deny, VIEW_CHANNEL) }) {
        return "everyone"
    }

    val entries = mutableListOf<String>()
    for (overwrite in overwrites) {
        if (hasPermission(overwrite.allow, VIEW_CHANNEL)) {
            entries.add("allow:${overwrite.id}")
        } else if (hasPermission(overwrite.deny, VIEW_CHANNEL)) {
            entries.add("deny:${overwrite.id}")
        }
    }
    entries.sort()

    return murmurhash32(entries.joinToString(",")).toString()
}
