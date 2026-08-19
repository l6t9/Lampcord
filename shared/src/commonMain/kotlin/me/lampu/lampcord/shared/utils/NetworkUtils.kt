package me.lampu.lampcord.shared.utils

expect suspend fun fetchUrlBytes(url: String): ByteArray?

expect fun base64Encode(bytes: ByteArray): String

expect suspend fun downloadToDownloads(url: String, filename: String): Boolean
