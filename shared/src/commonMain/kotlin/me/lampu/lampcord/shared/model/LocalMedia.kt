package me.lampu.lampcord.shared.model

data class LocalMedia(
    val id: Long,
    val uri: String,
    val name: String,
    val size: Long,
    val mimeType: String,
    val isVideo: Boolean = false,
    val duration: Long? = null
)
