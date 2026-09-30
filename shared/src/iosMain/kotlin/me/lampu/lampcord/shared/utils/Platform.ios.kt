package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.room.Room
import androidx.room.RoomDatabase
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.convert
import kotlinx.cinterop.usePinned
import me.lampu.lampcord.shared.database.AppDatabase
import me.lampu.lampcord.shared.model.LocalMedia
import platform.Foundation.NSApplicationSupportDirectory
import platform.Foundation.NSFileManager
import platform.Foundation.NSSearchPathForDirectoriesInDomains
import platform.Foundation.NSUserDomainMask
import platform.Foundation.timeIntervalSince1970
import platform.Foundation.NSDate
import platform.Foundation.NSUUID
import platform.UIKit.UIDevice
import platform.posix.access
import platform.posix.fclose
import platform.posix.fopen
import platform.posix.fread
import platform.posix.fseek
import platform.posix.fwrite
import platform.posix.getenv
import platform.posix.mkdir
import platform.posix.stat

actual fun getPlatformName(): String = "ios"

actual fun getCurrentTimeMillis(): Long = (NSDate().timeIntervalSince1970 * 1000).toLong()

actual fun randomUUID(): String = NSUUID().UUIDString()

actual fun getOsVersion(): String = UIDevice.currentDevice.systemVersion

actual fun getOsSdkVersion(): String = UIDevice.currentDevice.systemVersion

/**
 * Every device iOS ships to is arm64, and this value feeds the gateway identify payload where a
 * wrong architecture means the connection is rejected.
 */
actual fun getOsArch(): String = "arm64"

actual fun getDeviceName(): String = UIDevice.currentDevice.model

actual fun getCpuCoreCount(): Int = platform.posix.sysconf(platform.posix._SC_NPROCESSORS_ONLN).toInt()

actual fun getMemoryMemory(): Long =
    platform.posix.sysconf(platform.posix._SC_PHYS_PAGES) *
        platform.posix.sysconf(platform.posix._SC_PAGE_SIZE)


actual suspend fun getLocalMedia(): List<LocalMedia> = emptyList()

actual suspend fun getLocalFiles(): List<LocalMedia> = emptyList()

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = null

actual fun getDatabaseBuilder(): RoomDatabase.Builder<AppDatabase> =
    Room.databaseBuilder<AppDatabase>(name = "$appStorageDirectory/lampcord.db")

@OptIn(ExperimentalForeignApi::class)
private val appStorageDirectory: String by lazy {
    val root = NSSearchPathForDirectoriesInDomains(
        NSApplicationSupportDirectory,
        NSUserDomainMask,
        true
    ).firstOrNull() as? String ?: (getenv("HOME") ?: ".")
    val directory = "$root/lampcord"
    NSFileManager.defaultManager.createDirectoryAtPath(
        directory,
        withIntermediateDirectories = true,
        attributes = null,
        error = null
    )
    directory
}

actual fun getAppStoragePath(): String = appStorageDirectory

@OptIn(ExperimentalForeignApi::class)
private fun internalFile(name: String) = "$appStorageDirectory/$name"

@OptIn(ExperimentalForeignApi::class)
actual fun writeInternalFile(name: String, content: String) {
    writeBytes(internalFile(name), content.encodeToByteArray())
}

@OptIn(ExperimentalForeignApi::class)
actual fun writeInternalBytes(name: String, content: ByteArray) {
    writeBytes(internalFile(name), content)
}

@OptIn(ExperimentalForeignApi::class)
actual fun readInternalFile(name: String): String? =
    readBytes(internalFile(name))?.decodeToString()

@OptIn(ExperimentalForeignApi::class)
actual fun checkInternalFileExists(name: String): Boolean = stat(internalFile(name), null) == 0

@OptIn(ExperimentalForeignApi::class)
actual fun getInternalFilePath(name: String): String = internalFile(name)

actual fun showToast(text: String) {
    Logging.d("ui", text)
}

actual fun restartApp() {
    reloadTrigger.value += 1
}

@Composable
actual fun RequestMediaPermissions(onResult: (Boolean) -> Unit) {
    LaunchedEffect(Unit) {
        onResult(true)
    }
}

@OptIn(ExperimentalForeignApi::class)
internal fun writeBytes(path: String, bytes: ByteArray): Boolean {
    val file = fopen(path, "wb") ?: return false
    return try {
        val written = bytes.usePinned { pinned ->
            fwrite(pinned.addressOf(0), 1.convert(), bytes.size.convert(), file)
        }
        written.toInt() == bytes.size
    } catch (e: Exception) {
        Logging.w("files", "Unable to write $path", e)
        false
    } finally {
        fclose(file)
    }
}

@OptIn(ExperimentalForeignApi::class)
internal fun readBytes(path: String): ByteArray? {
    if (access(path, 0) != 0) return null
    val file = fopen(path, "rb") ?: return null
    return try {
        fseek(file, 0, SEEK_END)
        val size = platform.posix.ftell(file)
        if (size <= 0L) return null
        fseek(file, 0, SEEK_SET)
        val bytes = ByteArray(size.toInt())
        val read = bytes.usePinned { pinned ->
            fread(pinned.addressOf(0), 1.convert(), size.convert(), file)
        }
        if (read.toLong() == size) bytes else null
    } catch (e: Exception) {
        Logging.w("files", "Unable to read $path", e)
        null
    } finally {
        fclose(file)
    }
}

private const val SEEK_END = 2
private const val SEEK_SET = 0