package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import me.lampu.lampcord.shared.model.LocalMedia

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

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = null

@Composable
actual fun RequestMediaPermissions(onResult: (Boolean) -> Unit) {
    LaunchedEffect(Unit) {
        onResult(true)
    }
}
