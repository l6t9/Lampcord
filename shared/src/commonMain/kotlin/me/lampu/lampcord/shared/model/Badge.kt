package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class CustomBadge(
    val id: String,
    val name: String,
    val description: String? = null,
    val icon: String? = null // URL to icon
)

@Serializable
data class BadgeMapping(
    val badges: Map<String, CustomBadge> = emptyMap(),
    val users: Map<String, List<String>> = emptyMap() // userId -> list of badge ids
)
