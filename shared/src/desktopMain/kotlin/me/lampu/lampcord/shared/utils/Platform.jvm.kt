package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import me.lampu.lampcord.shared.model.LocalMedia
import androidx.room.Room
import androidx.room.RoomDatabase
import me.lampu.lampcord.shared.database.AppDatabase
import java.io.File

actual fun getPlatformName(): String {
    val os = System.getProperty("os.name").lowercase()
    return when {
        os.contains("win") -> "windows"
        os.contains("mac") -> "macos"
        os.contains("nix") || os.contains("nux") -> "linux"
        else -> "jvm"
    }
}

actual fun getCurrentTimeMillis(): Long = System.currentTimeMillis()

actual fun randomUUID(): String = java.util.UUID.randomUUID().toString()

actual fun getOsVersion(): String = System.getProperty("os.version") ?: "unknown"

actual fun getOsSdkVersion(): String = "24"

actual fun getOsArch(): String = System.getProperty("os.arch") ?: "unknown"

actual fun getDeviceName(): String = "Desktop"

actual fun getCpuCoreCount(): Int = Runtime.getRuntime().availableProcessors()

actual fun getMemoryMemory(): Long = Runtime.getRuntime().totalMemory() / (1024 * 1024)

actual suspend fun getLocalMedia(): List<LocalMedia> = emptyList()

actual suspend fun getLocalFiles(): List<LocalMedia> = emptyList()

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = null

actual fun getDatabaseBuilder(): RoomDatabase.Builder<AppDatabase> {
    val dbFile = File(getAppStoragePath(), "lampcord.db")
    return Room.databaseBuilder<AppDatabase>(
        name = dbFile.absolutePath,
    )
}

actual fun getAppStoragePath(): String {
    val os = getPlatformName()
    return when (os) {
        "windows" -> System.getenv("APPDATA") ?: System.getProperty("user.home")
        "macos" -> System.getProperty("user.home") + "/Library/Application Support"
        else -> System.getProperty("user.home") + "/.config"
    } + "/lampcord"
}

actual fun writeInternalFile(name: String, content: String) {
    val dir = java.io.File(getAppStoragePath())
    if (!dir.exists()) dir.mkdirs()
    java.io.File(dir, name).writeText(content)
}

actual fun writeInternalBytes(name: String, content: ByteArray) {
    val dir = java.io.File(getAppStoragePath())
    if (!dir.exists()) dir.mkdirs()
    java.io.File(dir, name).writeBytes(content)
}

actual fun readInternalFile(name: String): String? {
    val file = java.io.File(getAppStoragePath(), name)
    return if (file.exists()) file.readText() else null
}

actual fun checkInternalFileExists(name: String): Boolean {
    return java.io.File(getAppStoragePath(), name).exists()
}

actual fun getInternalFilePath(name: String): String {
    return java.io.File(getAppStoragePath(), name).absolutePath
}

actual fun showToast(text: String) {
    println("Toast: $text")
}

@Composable
actual fun RequestMediaPermissions(onResult: (Boolean) -> Unit) {
    LaunchedEffect(Unit) {
        onResult(true)
    }
}
