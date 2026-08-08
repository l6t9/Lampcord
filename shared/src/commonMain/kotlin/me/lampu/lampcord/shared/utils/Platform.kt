package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.model.LocalMedia

expect fun getPlatformName(): String

expect fun getCurrentTimeMillis(): Long

expect fun randomUUID(): String

expect fun getOsVersion(): String

expect fun getOsSdkVersion(): String

expect fun getOsArch(): String

expect fun getDeviceName(): String

expect fun getCpuCoreCount(): Int

expect fun getMemoryMemory(): Long

expect suspend fun getLocalMedia(): List<LocalMedia>

expect suspend fun getLocalMediaBytes(uri: String): ByteArray?
