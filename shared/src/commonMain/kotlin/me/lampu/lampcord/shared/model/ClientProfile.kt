package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class CustomProfile(
    val theme_colors: List<Int>? = null,
    val banner: String? = null,
    val accent_color: Int? = null
)

@Serializable
data class ClientProfileMapping(
    val users: Map<String, CustomProfile> = emptyMap()
)
