package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import me.lampu.lampcord.shared.model.LocalMedia
import androidx.room.RoomDatabase
import me.lampu.lampcord.shared.database.AppDatabase

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

expect suspend fun getLocalFiles(): List<LocalMedia>

expect suspend fun getLocalMediaBytes(uri: String): ByteArray?

expect fun getDatabaseBuilder(): RoomDatabase.Builder<AppDatabase>

expect fun getAppStoragePath(): String

expect fun writeInternalFile(name: String, content: String)

expect fun readInternalFile(name: String): String?

@Composable
expect fun RequestMediaPermissions(onResult: (Boolean) -> Unit)
