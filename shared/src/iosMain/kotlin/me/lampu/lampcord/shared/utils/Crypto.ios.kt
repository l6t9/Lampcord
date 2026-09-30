package me.lampu.lampcord.shared.utils

import kotlinx.cinterop.CPointer
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import kotlinx.cinterop.reinterpret
import kotlinx.cinterop.usePinned
import platform.CoreFoundation.CFDataCreate
import platform.CoreFoundation.CFDataGetBytes
import platform.CoreFoundation.CFDataRef
import platform.CoreFoundation.CFDataGetLength
import platform.CoreFoundation.CFDictionaryCreateMutable
import platform.CoreFoundation.CFDictionaryRef
import platform.CoreFoundation.CFDictionarySetValue
import platform.CoreFoundation.CFRangeMake
import platform.CoreFoundation.CFRelease
import platform.CoreFoundation.CFStringRef
import platform.CoreFoundation.CFTypeRef
import platform.CoreFoundation.kCFBooleanFalse
import platform.CoreFoundation.kCFTypeDictionaryKeyCallBacks
import platform.CoreFoundation.kCFTypeDictionaryValueCallBacks
import platform.Security.SecKeyCopyExternalRepresentation
import platform.Security.SecKeyCopyPublicKey
import platform.Security.SecKeyCreateDecryptedData
import platform.Security.SecKeyCreateRandomKey
import platform.Security.SecKeyRef
import platform.Security.kSecAttrIsPermanent
import platform.Security.kSecAttrKeyType
import platform.Security.kSecAttrKeyTypeRSA
import platform.Security.kSecKeyAlgorithmRSAEncryptionOAEPSHA256
import platform.Security.kSecPrivateKeyAttrs
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

@OptIn(ExperimentalForeignApi::class)
actual class RSAKeyPair internal constructor(
    private val privateKey: SecKeyRef?,
    private val publicKey: SecKeyRef?
) {
    @OptIn(ExperimentalEncodingApi::class)
    actual fun getPublicKeyBase64(): String {
        val key = publicKey ?: return ""
        val external = SecKeyCopyExternalRepresentation(key, null) ?: return ""
        val bytes = external.copyToByteArray()
        CFRelease(external)
        return if (bytes == null) "" else Base64.encode(bytes)
    }

    actual fun decrypt(encryptedData: ByteArray): ByteArray {
        val key = privateKey ?: return byteArrayOf()
        if (encryptedData.isEmpty()) return byteArrayOf()

        val cipher = memScoped {
            encryptedData.usePinned { pinned ->
                CFDataCreate(null, pinned.addressOf(0).reinterpret(), encryptedData.size.toLong())
            }
        } ?: return byteArrayOf()

        val decrypted = SecKeyCreateDecryptedData(
            key,
            kSecKeyAlgorithmRSAEncryptionOAEPSHA256,
            cipher,
            null
        )
        CFRelease(cipher)
        if (decrypted == null) return byteArrayOf()

        val out = decrypted.copyToByteArray() ?: byteArrayOf()
        CFRelease(decrypted)
        return out
    }
}

@OptIn(ExperimentalForeignApi::class)
actual object CryptoUtils {
    @OptIn(ExperimentalForeignApi::class)
    actual fun generateRSAKeyPair(): RSAKeyPair {
        val privateAttributes = attributes(kSecAttrIsPermanent to kCFBooleanFalse)
        val keyAttributes = attributes(
            kSecAttrKeyType to kSecAttrKeyTypeRSA,
            kSecPrivateKeyAttrs to privateAttributes
        )

        val privateKey = SecKeyCreateRandomKey(keyAttributes, null)
        CFRelease(keyAttributes)
        CFRelease(privateAttributes)

        if (privateKey == null) {
            Logging.e("crypto", "SecKeyCreateRandomKey failed to produce an RSA key")
            return RSAKeyPair(null, null)
        }
        return RSAKeyPair(privateKey, SecKeyCopyPublicKey(privateKey))
    }

    actual fun sha256(data: ByteArray): ByteArray = me.lampu.lampcord.shared.utils.sha256(data)

    @OptIn(ExperimentalForeignApi::class)
    private fun attributes(vararg pairs: Pair<CFStringRef?, CFTypeRef?>): CFDictionaryRef {
        val dictionary = CFDictionaryCreateMutable(
            null,
            pairs.size.toLong(),
            kCFTypeDictionaryKeyCallBacks.ptr,
            kCFTypeDictionaryValueCallBacks.ptr
        ) ?: error("Unable to allocate a CoreFoundation dictionary")
        pairs.forEach { (key, value) -> CFDictionarySetValue(dictionary, key, value) }
        return dictionary
    }
}

@OptIn(ExperimentalForeignApi::class)
private fun CFDataRef.copyToByteArray(): ByteArray? {
    val length = CFDataGetLength(this)
    if (length <= 0L) return null
    val bytes = ByteArray(length.toInt())
    bytes.usePinned { pinned ->
        CFDataGetBytes(this, platform.CoreFoundation.CFRangeMake(0, length), pinned.addressOf(0).reinterpret())
    }
    return bytes
}
