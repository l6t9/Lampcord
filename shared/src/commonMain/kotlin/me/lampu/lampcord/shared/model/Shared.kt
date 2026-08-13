package me.lampu.lampcord.shared.model

import kotlinx.serialization.json.*

fun JsonElement?.asSnowflake(): String? = when {
    this == null -> null
    this is JsonPrimitive -> if (this.isString) content else contentOrNull
    else -> null
}

