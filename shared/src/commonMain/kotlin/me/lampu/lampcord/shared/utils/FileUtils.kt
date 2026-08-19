package me.lampu.lampcord.shared.utils

import kotlin.math.min

fun sanitizeFilename(name: String): String {
    var n = name.trim()
    if (n.isEmpty()) return "download"
    // Remove any path separators and control characters
    n = n.replace(Regex("[\\\\/:*?\"<>|\\r\\n\\t]"), "_")
    // Collapse multiple dots
    n = n.replace(Regex("\u002e{2,}"), ".")
    // Limit length
    if (n.length > 128) n = n.take(128)
    return n
}

expect suspend fun ensureUniqueDownloadFilename(desiredName: String): String
expect fun openDownloadsFolderAndSelect(filename: String)
