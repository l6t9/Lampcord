package me.lampu.lampcord.shared.utils

actual fun getPlatformName(): String = "android"

actual fun getCurrentTimeMillis(): Long = System.currentTimeMillis()

actual fun randomUUID(): String = java.util.UUID.randomUUID().toString()
