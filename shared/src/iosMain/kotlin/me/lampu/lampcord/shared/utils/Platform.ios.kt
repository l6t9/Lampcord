package me.lampu.lampcord.shared.utils

import platform.Foundation.NSDate
import platform.Foundation.timeIntervalSince1970
import platform.Foundation.NSUUID
import platform.UIKit.UIDevice

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import me.lampu.lampcord.shared.model.LocalMedia
import androidx.room.Room
import androidx.room.RoomDatabase
import me.lampu.lampcord.shared.database.AppDatabase

actual fun getPlatformName(): String = "ios"

actual fun getCurrentTimeMillis(): Long = (NSDate().timeIntervalSince1970 * 1000).toLong()

actual fun randomUUID(): String = NSUUID().UUIDString()

actual fun getOsVersion(): String = UIDevice.currentDevice.systemVersion

actual fun getOsSdkVersion(): String = UIDevice.currentDevice.systemVersion

actual fun getOsArch(): String = "arm64" // Default for iOS

actual fun getDeviceName(): String = UIDevice.currentDevice.model

actual fun getCpuCoreCount(): Int = 8 // Default for common iPhones

actual fun getMemoryMemory(): Long = 4096 // Default for common iPhones

actual suspend fun getLocalMedia(): List<LocalMedia> = emptyList()

actual suspend fun getLocalFiles(): List<LocalMedia> = emptyList()

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = null

actual fun getDatabaseBuilder(): RoomDatabase.Builder<AppDatabase> {
    val dbFile = NSHomeDirectory() + "/Documents/lampcord.db"
    return Room.databaseBuilder<AppDatabase>(
        name = dbFile,
        factory = { AppDatabase::class.instantiateImpl() }
    )
}

actual fun getAppStoragePath(): String = "" // Placeholder for iOS

actual fun writeInternalFile(name: String, content: String) {}

actual fun writeInternalBytes(name: String, content: ByteArray) {}

actual fun readInternalFile(name: String): String? = null

actual fun checkInternalFileExists(name: String): Boolean = false

actual fun getInternalFilePath(name: String): String = ""

actual fun showToast(text: String) {
    println("Toast: $text")
}

actual fun restartApp() {
    platform.posix.exit(0)
}

@Composable
actual fun RequestMediaPermissions(onResult: (Boolean) -> Unit) {
    LaunchedEffect(Unit) {
        onResult(true)
    }
}
