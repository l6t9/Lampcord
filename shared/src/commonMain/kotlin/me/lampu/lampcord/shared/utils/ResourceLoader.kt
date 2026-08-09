package me.lampu.lampcord.shared.utils

expect object ResourceLoader {
    fun readText(path: String): String?
    fun readBytes(path: String): ByteArray?
}
