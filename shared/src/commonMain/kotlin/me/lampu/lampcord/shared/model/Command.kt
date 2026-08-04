package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class ApplicationCommand(
    val id: String,
    val application_id: String,
    val version: String? = null,
    val type: Int = 1,
    val name: String,
    val description: String? = null,
    val options: List<ApplicationCommandOption>? = null,
    val guild_id: String? = null
)

@Serializable
data class ApplicationCommandOption(
    val type: Int,
    val name: String,
    val description: String,
    val required: Boolean = false,
    val choices: List<ApplicationCommandOptionChoice>? = null,
    val options: List<ApplicationCommandOption>? = null,
    val channel_types: List<Int>? = null,
    val min_value: JsonElement? = null,
    val max_value: JsonElement? = null,
    val min_length: Int? = null,
    val max_length: Int? = null,
    val autocomplete: Boolean? = null
)

@Serializable
data class ApplicationCommandOptionChoice(
    val name: String,
    val value: JsonElement
)

@Serializable
data class ApplicationCommandIndex(
    val applications: List<Application>,
    val application_commands: List<ApplicationCommand>
)

@Serializable
data class Application(
    val id: String,
    val name: String,
    val icon: String? = null,
    val description: String? = null,
    val bot: User? = null
)

@Serializable
data class InteractionRequest(
    val type: Int,
    val application_id: String,
    val guild_id: String? = null,
    val channel_id: String,
    val session_id: String,
    val data: InteractionData,
    val nonce: String? = null
)

@Serializable
data class InteractionData(
    val id: String,
    val name: String,
    val type: Int = 1,
    val version: String? = null,
    val options: List<InteractionOption>? = null
)

@Serializable
data class InteractionOption(
    val type: Int,
    val name: String,
    val value: JsonElement? = null,
    val focused: Boolean? = null,
    val options: List<InteractionOption>? = null
)
