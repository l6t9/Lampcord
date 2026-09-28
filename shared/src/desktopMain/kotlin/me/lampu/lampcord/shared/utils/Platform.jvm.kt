package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import me.lampu.lampcord.shared.model.LocalMedia
import androidx.room.Room
import androidx.room.RoomDatabase
import me.lampu.lampcord.shared.database.AppDatabase
import java.io.File
import java.net.URI
import java.net.URL
import java.util.Base64

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

actual fun restartApp() {
    reloadTrigger.value++
}

@Composable
actual fun RequestMediaPermissions(onResult: (Boolean) -> Unit) {
    LaunchedEffect(Unit) {
        onResult(true)
    }
}

suspend fun safeReadUrl(url: String): ByteArray? = try {
    with(java.lang.Runnable { }) {
        URI(url).toURL().readBytes()
    }
} catch (e: Exception) { null }

actual suspend fun fetchUrlBytes(url: String): ByteArray? = try {
    URI(url).toURL().readBytes()
} catch (e: Exception) { null }

actual fun base64Encode(bytes: ByteArray): String = Base64.getEncoder().encodeToString(bytes)

actual suspend fun downloadToDownloads(url: String, filename: String): Boolean {
    return try {
        val bytes = fetchUrlBytes(url) ?: return false
        val downloads = java.io.File(System.getProperty("user.home"), "Downloads")
        if (!downloads.exists()) downloads.mkdirs()
        val out = java.io.File(downloads, filename)
        out.writeBytes(bytes)
        true
    } catch (e: Exception) {
        false
    }
}

actual suspend fun ensureUniqueDownloadFilename(desiredName: String): String {
    return try {
        val downloads = java.io.File(System.getProperty("user.home"), "Downloads")
        if (!downloads.exists()) downloads.mkdirs()
        var base = desiredName
        var ext = ""
        val idx = desiredName.lastIndexOf('.')
        if (idx > 0) {
            base = desiredName.substring(0, idx)
            ext = desiredName.substring(idx)
        }
        var candidate = desiredName
        var i = 1
        while (java.io.File(downloads, candidate).exists()) {
            candidate = "$base ($i)$ext"
            i++
            if (i > 1000) break
        }
        candidate
    } catch (e: Exception) {
        desiredName
    }
}

actual fun openDownloadsFolderAndSelect(filename: String) {
    try {
        val downloads = java.io.File(System.getProperty("user.home"), "Downloads")
        if (!downloads.exists()) downloads.mkdirs()
        val f = java.io.File(downloads, filename)
        val desktop = java.awt.Desktop.getDesktop()
        if (f.exists()) {
            desktop.open(f.parentFile)
        } else {
            desktop.open(downloads)
        }
    } catch (e: Exception) {
    }
}
