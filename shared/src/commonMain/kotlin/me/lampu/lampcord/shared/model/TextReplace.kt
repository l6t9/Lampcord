package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class TextReplaceRule(
    val id: String,
    val pattern: String,
    val replacement: String,
    val isRegex: Boolean = false,
    val ignoreCase: Boolean = true,
    val enabled: Boolean = true,
    val matchSent: Boolean = false,
    val matchUnsent: Boolean = true
)
