package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class CustomProfile(
    @SerialName("u") val user_id: String? = null,
    @SerialName("c") val theme_colors: List<Int>? = null,
    @SerialName("b") val banner: String? = null,
    @SerialName("a") val accent_color: Int? = null,
    @SerialName("p") val avatar: String? = null
)

@Serializable
data class ClientProfileMapping(
    val users: Map<String, CustomProfile> = emptyMap()
)
