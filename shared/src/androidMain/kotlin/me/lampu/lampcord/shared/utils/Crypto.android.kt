package me.lampu.lampcord.shared.utils

import java.security.KeyPairGenerator
import java.security.MessageDigest
import java.security.PrivateKey
import java.security.PublicKey
import javax.crypto.Cipher
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

actual class RSAKeyPair(private val publicKey: PublicKey, private val privateKey: PrivateKey) {
    @OptIn(ExperimentalEncodingApi::class)
    actual fun getPublicKeyBase64(): String {
        val encoded = publicKey.encoded
        // SubjectPublicKeyInfo (DER)
        return Base64.Default.encode(encoded)
            .replace("\n", "")
            .replace("\r", "")
    }

    actual fun decrypt(encryptedData: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("RSA/ECB/PKCS1Padding")
        cipher.init(Cipher.DECRYPT_MODE, privateKey)
        return cipher.doFinal(encryptedData)
    }
}

actual object CryptoUtils {
    actual fun generateRSAKeyPair(): RSAKeyPair {
        val kpg = KeyPairGenerator.getInstance("RSA")
        kpg.initialize(2048)
        val kp = kpg.generateKeyPair()
        return RSAKeyPair(kp.public, kp.private)
    }

    actual fun sha256(data: ByteArray): ByteArray {
        val digest = MessageDigest.getInstance("SHA-256")
        return digest.digest(data)
    }
}
