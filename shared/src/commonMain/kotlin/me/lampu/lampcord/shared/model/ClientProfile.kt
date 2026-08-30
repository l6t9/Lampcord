package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class CustomProfile(
    val user_id: String? = null,
    val theme_colors: List<Int>? = null,
    val banner: String? = null,
    val accent_color: Int? = null,
    val avatar: String? = null
)

@Serializable
data class ClientProfileMapping(
    val users: Map<String, CustomProfile> = emptyMap()
)
