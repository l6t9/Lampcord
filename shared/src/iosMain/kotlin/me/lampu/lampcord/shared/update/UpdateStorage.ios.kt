package me.lampu.lampcord.shared.update

actual object UpdateStorage {
    actual fun store(version: String, target: UpdateTarget, bytes: ByteArray): String = ""
    actual fun clear() = Unit
}
