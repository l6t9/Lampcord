package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Relationship(
    val id: String? = null,
    val type: Int? = null,
    val user: User? = null,
    val user_id: String? = null
)
