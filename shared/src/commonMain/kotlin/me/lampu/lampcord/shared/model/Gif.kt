package me.lampu.lampcord.shared.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient

@Serializable
data class Gif(
    val src: String,
    val url: String,
    val width: Int,
    val height: Int,
    val preview: String? = null,
    @SerialName("gif_src") val gifSrc: String? = null,
    @Transient val isVideo: Boolean = false
)

@Serializable
data class GifCategory(
    val name: String,
    val src: String
)

@Serializable
data class TrendingGifCategoriesResponse(
    val categories: List<GifCategory>,
    val gifs: List<Gif> = emptyList()
)
