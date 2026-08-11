package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Gif(
    val src: String,
    val url: String,
    val width: Int,
    val height: Int
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
