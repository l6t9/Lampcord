package me.lampu.lampcord.shared.update

import java.io.File

actual fun runningAppImagePath(): String? =
    System.getenv("APPIMAGE")?.takeIf { it.isNotBlank() }?.let(::File)?.takeIf { it.isFile }?.absolutePath
