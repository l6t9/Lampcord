package me.lampu.lampcord.shared.update

import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.readAvailable
import kotlinx.coroutines.yield
import okio.HashingSink
import okio.blackholeSink
import okio.buffer

internal const val CHUNK_SIZE = 64 * 1024

sealed interface StoreOutcome {
    data class Stored(val path: String) : StoreOutcome
    data object SizeMismatch : StoreOutcome
    data object DigestMismatch : StoreOutcome
    data class Failed(val reason: String) : StoreOutcome
}

interface UpdateSink {
    val path: String
    fun write(source: ByteArray, offset: Int, length: Int)
    fun commit()
    fun abort()
}

expect object UpdateStorage {
    fun open(version: String, target: UpdateTarget): UpdateSink
    fun clear()
}

internal fun fileNameFor(version: String, target: UpdateTarget): String =
    "lampcord-${target.assetInfix ?: target.name.lowercase()}-$version.${target.extension}"

internal val DIGEST_PATTERN = Regex("^sha256:[0-9a-f]{64}$", RegexOption.IGNORE_CASE)

internal fun normalizeDigest(digest: String?): String? =
    digest?.lowercase()?.takeIf { DIGEST_PATTERN.matches(it) }?.removePrefix("sha256:")

internal suspend fun storeDownload(
    version: String,
    target: UpdateTarget,
    expectedSize: Long,
    advertisedDigest: String?,
    source: ByteReadChannel,
): StoreOutcome {
    val expectedDigest = normalizeDigest(advertisedDigest) ?: return StoreOutcome.DigestMismatch

    val sink = UpdateStorage.open(version, target)
    val hasher = HashingSink.sha256(blackholeSink())
    val digest = hasher.buffer()
    val chunk = ByteArray(CHUNK_SIZE)
    var written = 0L

    return try {
        while (true) {
            val read = source.readAvailable(chunk, 0, chunk.size)
            if (read < 0) break
            if (read == 0) {
                yield()
                continue
            }
            written += read
            sink.write(chunk, 0, read)
            digest.write(chunk, 0, read)
        }
        // The buffered sink holds the tail of the last chunk, so it has to reach the hasher
        // before the digest is taken.
        digest.flush()

        when {
            written != expectedSize -> {
                sink.abort()
                StoreOutcome.SizeMismatch
            }

            hasher.hash.hex() != expectedDigest -> {
                sink.abort()
                StoreOutcome.DigestMismatch
            }

            else -> {
                sink.commit()
                StoreOutcome.Stored(sink.path)
            }
        }
    } catch (e: Exception) {
        sink.abort()
        StoreOutcome.Failed(e.message ?: "The download failed")
    }
}