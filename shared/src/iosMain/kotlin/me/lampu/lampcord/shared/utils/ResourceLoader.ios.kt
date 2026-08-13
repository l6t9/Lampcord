package me.lampu.lampcord.shared.utils

import platform.Foundation.NSBundle
import platform.Foundation.NSData
import platform.Foundation.toByteArray

actual object ResourceLoader {
    actual fun readText(path: String): String? {
        return readBytes(path)?.decodeToString()
    }

    actual fun readBytes(path: String): ByteArray? {
        val bundle = NSBundle.mainBundle
        val resourcePath = bundle.resourcePath ?: bundle.bundlePath
        return NSData.dataWithContentsOfFile("$resourcePath/$path")?.toByteArray()
    }
}
