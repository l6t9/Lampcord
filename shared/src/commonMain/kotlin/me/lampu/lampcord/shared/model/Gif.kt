package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class Gif(
    val src: String,
    val url: String,
    val width: Int,
    val height: Int,
    val preview: String? = null,
    @SerialName("gif_src") val gifSrc: String? = null
)

@Serializable
data class GifCategory(
    val name: String,
    val src: String
)

@Serializable
data class TrendingGifCategoriesResponse(
    val categories: List<GifCategory>
)
