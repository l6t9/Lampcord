package com.example.lampcord.shared.model

import kotlinx.serialization.json.*

fun JsonElement?.asSnowflake(): String? = when {
    this == null -> null
    this is JsonPrimitive -> if (this.isString) content else contentOrNull
    else -> null
}

fun Channel.lastMessageId(): String? = last_message_id.asSnowflake()
fun ReadState.lastMessageId(): String? = last_message_id.asSnowflake()
