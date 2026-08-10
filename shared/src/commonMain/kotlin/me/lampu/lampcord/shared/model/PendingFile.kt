package me.lampu.lampcord.shared.model

data class PendingFile(
    val name: String,
    val data: ByteArray,
    val uri: String? = null
)
