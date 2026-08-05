package me.lampu.lampcord.shared.utils

actual class RSAKeyPair {
    actual fun getPublicKeyBase64(): String = ""
    actual fun decrypt(encryptedData: ByteArray): ByteArray = byteArrayOf()
}

actual object CryptoUtils {
    actual fun generateRSAKeyPair(): RSAKeyPair = RSAKeyPair()
    actual fun sha256(data: ByteArray): ByteArray = byteArrayOf()
}
