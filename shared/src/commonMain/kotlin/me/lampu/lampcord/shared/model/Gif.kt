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
    // Favorite GIF protobuf entries identify whether the stored source is a
    // video even when the URL has no recognizable file extension.
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
    // Discord includes representative GIF results alongside the category
    // names. They provide reliable still previews when Reduced Motion is on.
    val gifs: List<Gif> = emptyList()
)
