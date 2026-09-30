package me.lampu.lampcord.shared.utils

import io.ktor.client.HttpClient
import io.ktor.client.engine.darwin.Darwin
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsBytes
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.coroutines.IO
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import platform.Foundation.NSDocumentDirectory
import platform.Foundation.NSFileManager
import platform.Foundation.NSSearchPathForDirectoriesInDomains
import platform.Foundation.NSUserDomainMask
import platform.posix.access
import platform.posix.getenv
import platform.posix.stat
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

@OptIn(ExperimentalForeignApi::class)
private fun downloadsDirectory(): String {
    val documents = NSSearchPathForDirectoriesInDomains(
        NSDocumentDirectory,
        NSUserDomainMask,
        true
    ).firstOrNull() as? String ?: (getenv("HOME") ?: ".")
    val downloads = "$documents/Downloads"
    NSFileManager.defaultManager.createDirectoryAtPath(
        downloads,
        withIntermediateDirectories = true,
        attributes = null,
        error = null
    )
    return downloads
}

@OptIn(ExperimentalForeignApi::class)
private fun fileExists(path: String): Boolean = stat(path, null) == 0

actual suspend fun fetchUrlBytes(url: String): ByteArray? = withContext(Dispatchers.IO) {
    runCatching {
        HttpClient(Darwin).use { client -> client.get(url).bodyAsBytes() }
    }.onFailure { Logging.w("net", "Unable to fetch $url", it) }.getOrNull()
}

@OptIn(ExperimentalEncodingApi::class)
actual fun base64Encode(bytes: ByteArray): String = Base64.encode(bytes)

actual suspend fun downloadToDownloads(url: String, filename: String): Boolean =
    withContext(Dispatchers.IO) {
        val bytes = fetchUrlBytes(url) ?: return@withContext false
        writeBytes(downloadsDirectory() + "/" + filename, bytes)
    }

actual suspend fun ensureUniqueDownloadFilename(desiredName: String): String =
    withContext(Dispatchers.IO) {
        val directory = downloadsDirectory()
        val index = desiredName.lastIndexOf('.')
        val base = if (index > 0) desiredName.substring(0, index) else desiredName
        val extension = if (index > 0) desiredName.substring(index) else ""
        var candidate = desiredName
        var suffix = 1
        while (fileExists("$directory/$candidate") && suffix <= 1000) {
            candidate = "$base ($suffix)$extension"
            suffix++
        }
        candidate
    }

actual fun openDownloadsFolderAndSelect(filename: String) {
    Logging.i("net", "Downloads are stored at ${downloadsDirectory()}/$filename")
}

