package me.lampu.lampcord.shared.utils

import java.io.InputStream

actual object ResourceLoader {
    actual fun readText(path: String): String? {
        return readBytes(path)?.decodeToString()
    }

    actual fun readBytes(path: String): ByteArray? {
        val stream: InputStream? = try {
            AndroidContextProvider.applicationContext.assets.open(path)
        } catch (e: Exception) {
            try {
                AndroidContextProvider.applicationContext.classLoader?.getResourceAsStream(path)
                    ?: ResourceLoader::class.java.getResourceAsStream("/$path")
                    ?: ResourceLoader::class.java.getResourceAsStream(path)
            } catch (e2: Exception) {
                null
            }
        }
        
        return stream?.use { it.readBytes() }
    }
}
