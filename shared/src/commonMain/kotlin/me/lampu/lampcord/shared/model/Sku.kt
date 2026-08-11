package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Sku(
    val id: String,
    val type: Int,
    val application_id: String,
    val name: String,
    val flags: Int,
    val premium: Boolean = false
)
