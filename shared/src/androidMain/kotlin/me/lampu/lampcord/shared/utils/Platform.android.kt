package me.lampu.lampcord.shared.utils

import android.os.Build

actual fun getPlatformName(): String = "android"

actual fun getCurrentTimeMillis(): Long = System.currentTimeMillis()

actual fun randomUUID(): String = java.util.UUID.randomUUID().toString()

actual fun getOsVersion(): String = Build.VERSION.RELEASE

actual fun getOsArch(): String = Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown"

actual fun getDeviceName(): String = "${Build.MANUFACTURER} ${Build.MODEL}"
