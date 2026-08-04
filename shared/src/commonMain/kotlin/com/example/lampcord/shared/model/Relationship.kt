package com.example.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Relationship(
    val id: String,
    val type: Int,
    val user: User
)
