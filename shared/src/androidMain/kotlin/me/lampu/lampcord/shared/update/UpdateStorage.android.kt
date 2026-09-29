package me.lampu.lampcord.shared.update

import me.lampu.lampcord.shared.utils.AndroidContextProvider
import java.io.File

actual object UpdateStorage {
    private val directory: File by lazy {
        File(AndroidContextProvider.applicationContext.cacheDir, "updates").apply { mkdirs() }
    }

    actual fun store(version: String, target: UpdateTarget, bytes: ByteArray): String {
        val file = File(directory, fileNameFor(version, target))
        file.writeBytes(bytes)
        return file.absolutePath
    }

    actual fun clear() {
        directory.listFiles()?.forEach { it.delete() }
    }
}
