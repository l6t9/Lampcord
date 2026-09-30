package me.lampu.lampcord.shared.utils

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.convert
import kotlinx.cinterop.usePinned
import platform.Foundation.NSBundle
import platform.Foundation.NSData
import platform.Foundation.dataWithContentsOfFile
import platform.posix.memcpy

@OptIn(ExperimentalForeignApi::class)
actual object ResourceLoader {

    actual fun readText(path: String): String? = readBytes(path)?.decodeToString()

    actual fun readBytes(path: String): ByteArray? {
        val root = NSBundle.mainBundle.resourcePath ?: NSBundle.mainBundle.bundlePath
        return readFile("$root/$path")
    }

    private fun readFile(path: String): ByteArray? {
        val data: NSData = NSData.dataWithContentsOfFile(path) ?: return null
        val length = data.length.toLong()
        if (length <= 0L) return null
        val bytes = ByteArray(length.toInt())
        val source = data.bytes ?: return null
        bytes.usePinned { pinned -> memcpy(pinned.addressOf(0), source, length.convert()) }
        return bytes
    }
}