package me.lampu.lampcord.shared.update

import me.lampu.lampcord.shared.utils.AndroidContextProvider
import java.io.File

actual object UpdateStorage {
    private val directory: File by lazy {
        File(AndroidContextProvider.applicationContext.cacheDir, "updates").apply { mkdirs() }
    }

    actual fun open(version: String, target: UpdateTarget): UpdateSink =
        FileUpdateSink(File(directory, fileNameFor(version, target)))

    actual fun clear() {
        directory.listFiles()?.forEach { it.delete() }
    }
}

private class FileUpdateSink(private val file: File) : UpdateSink {
    private val stream = file.outputStream().buffered(CHUNK_SIZE)

    override val path: String get() = file.absolutePath

    override fun write(source: ByteArray, offset: Int, length: Int) {
        stream.write(source, offset, length)
    }

    override fun commit() {
        stream.close()
    }

    override fun abort() {
        runCatching { stream.close() }
        file.delete()
    }
}