package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class VoiceGatewayPayload(
    val op: Int,
    val d: JsonElement? = null
)

@Serializable
data class VoiceIdentify(
    val server_id: String,
    val user_id: String,
    val session_id: String,
    val token: String,
    val video: Boolean = true,
    val streams: List<VoiceStream>? = null
)

@Serializable
data class VoiceStream(
    val type: String,
    val rid: String,
    val quality: Int
)

@Serializable
data class VoiceReady(
    val ssrc: Int,
    val ip: String,
    val port: Int,
    val modes: List<String>
)

@Serializable
data class VoiceSelectProtocol(
    val protocol: String = "udp",
    val data: VoiceSelectProtocolData
)

@Serializable
data class VoiceSelectProtocolData(
    val address: String,
    val port: Int,
    val mode: String
)

@Serializable
data class VoiceSessionDescription(
    val mode: String,
    val secret_key: List<Int>
)
