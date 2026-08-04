package com.example.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Sticker(
    val id: String,
    val pack_id: String? = null,
    val name: String,
    val description: String? = null,
    val tags: String? = null,
    val type: Int,
    val format_type: Int,
    val available: Boolean? = null,
    val guild_id: String? = null,
    val user: User? = null,
    val sort_value: Int? = null
)

@Serializable
data class StickerItem(
    val id: String,
    val name: String,
    val format_type: Int
)
