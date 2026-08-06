package me.lampu.lampcord.shared.utils

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

actual fun getOsArch(): String = System.getProperty("os.arch") ?: "unknown"

actual fun getDeviceName(): String = "Desktop"
