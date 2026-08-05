package me.lampu.lampcord.shared.utils

expect class RSAKeyPair {
    fun getPublicKeyBase64(): String
    fun decrypt(encryptedData: ByteArray): ByteArray
}

expect object CryptoUtils {
    fun generateRSAKeyPair(): RSAKeyPair
    fun sha256(data: ByteArray): ByteArray
}
