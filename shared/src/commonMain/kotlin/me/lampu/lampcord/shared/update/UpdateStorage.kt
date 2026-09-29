package me.lampu.lampcord.shared.update

import okio.ByteString.Companion.toByteString

expect object UpdateStorage {
    fun store(version: String, target: UpdateTarget, bytes: ByteArray): String
    fun clear()
}

fun downloadMatchesDigest(bytes: ByteArray, digest: String?): Boolean {
    val normalized = digest?.lowercase()?.takeIf { DIGEST_PATTERN.matches(it) } ?: return false
    return bytes.toByteString().sha256().hex() == normalized.removePrefix("sha256:")
}

internal fun fileNameFor(version: String, target: UpdateTarget): String =
    "lampcord-${target.assetInfix ?: target.name.lowercase()}-$version.${target.extension}"

internal val DIGEST_PATTERN = Regex("^sha256:[0-9a-f]{64}$", RegexOption.IGNORE_CASE)
