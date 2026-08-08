package me.lampu.lampcord.shared.utils

import platform.Foundation.NSDate
import platform.Foundation.timeIntervalSince1970
import platform.Foundation.NSUUID
import platform.UIKit.UIDevice

import me.lampu.lampcord.shared.model.LocalMedia

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

actual suspend fun getLocalMediaBytes(uri: String): ByteArray? = null
