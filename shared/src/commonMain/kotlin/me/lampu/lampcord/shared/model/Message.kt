package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.boolean
import kotlinx.serialization.json.jsonPrimitive

object NonceSerializer : KSerializer<String?> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("nonce", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): String? {
        return when (val input = (decoder as? JsonDecoder)?.decodeJsonElement()) {
            is JsonPrimitive -> input.contentOrNull
            else -> null
        }
    }

    @OptIn(kotlinx.serialization.ExperimentalSerializationApi::class)
    override fun serialize(encoder: Encoder, value: String?) {
        if (value == null) {
            encoder.encodeNull()
        } else {
            encoder.encodeString(value)
        }
    }
}

@Serializable
data class Message(
    val id: String,
    val channel_id: String,
    val author: User? = null,
    val content: String = "",
    val timestamp: String = "",
    val edited_timestamp: String? = null,
    val tts: Boolean = false,
    val mention_everyone: Boolean = false,
    val mentions: List<User> = emptyList(),
    val mention_roles: List<String> = emptyList(),
    val attachments: List<Attachment> = emptyList(),
    val embeds: List<Embed> = emptyList(),
    val reactions: List<MessageReaction>? = null,
    @Serializable(with = NonceSerializer::class)
    val nonce: String? = null,
    val pinned: Boolean = false,
    val webhook_id: String? = null,
    val type: Int? = null,
    val member: Member? = null,
    val guild_id: String? = null,
    val referenced_message: Message? = null,
    val message_reference: MessageReference? = null,
    val sticker_items: List<StickerItem>? = null,
    val message_snapshots: List<MessageSnapshot>? = null,
    val poll: Poll? = null,
    val components: List<MessageComponent>? = null,
    val interaction: MessageInteraction? = null,
    val interaction_metadata: MessageInteractionMetadata? = null,
    val isPending: Boolean = false,
    val sendError: String? = null,
    val hit: Boolean = false,
    val isDeleted: Boolean = false,
    val oldContent: String? = null
) {
    fun merge(data: kotlinx.serialization.json.JsonObject): Message {
        return this.copy(
            content = data["content"]?.jsonPrimitive?.content ?: content,
            pinned = data["pinned"]?.jsonPrimitive?.boolean ?: pinned,
            edited_timestamp = data["edited_timestamp"]?.jsonPrimitive?.contentOrNull ?: edited_timestamp
        )
    }
}

@Serializable
data class MessageReference(
    val message_id: String? = null,
    val channel_id: String? = null,
    val guild_id: String? = null,
    val fail_if_not_exists: Boolean? = null,
    val type: Int? = null
)

@Serializable
data class MessageSnapshot(
    val message: SnapshotMessage
)

@Serializable
data class SnapshotMessage(
    val author: User? = null,
    val content: String = "",
    val timestamp: String = "",
    val edited_timestamp: String? = null,
    val attachments: List<Attachment> = emptyList(),
    val embeds: List<Embed> = emptyList(),
    val type: Int? = null,
    val flags: Int? = null,
    val sticker_items: List<StickerItem>? = null,
    val components: List<MessageComponent>? = null
)

@Serializable
data class Poll(
    val question: PollMedia,
    val answers: List<PollAnswer>,
    val expiry: String? = null,
    val allow_multiselect: Boolean = false,
    val layout_type: Int = 1,
    val results: PollResults? = null
)

@Serializable
data class PollMedia(
    val text: String? = null,
    val emoji: Emoji? = null
)

@Serializable
data class PollAnswer(
    val answer_id: Int,
    val poll_media: PollMedia
)

@Serializable
data class PollResults(
    val is_finalized: Boolean,
    val answer_counts: List<PollAnswerCount>
)

@Serializable
data class PollAnswerCount(
    val id: Int,
    val count: Int,
    val me_voted: Boolean
)

@Serializable
data class MessageComponent(
    val type: Int,
    val components: List<MessageComponent>? = null,
    val style: Int? = null,
    val label: String? = null,
    val emoji: Emoji? = null,
    val custom_id: String? = null,
    val url: String? = null,
    val disabled: Boolean? = null,
    val placeholder: String? = null,
    val min_values: Int? = null,
    val max_values: Int? = null,
    val options: List<SelectMenuOption>? = null
)

@Serializable
data class SelectMenuOption(
    val label: String,
    val value: String,
    val description: String? = null,
    val emoji: Emoji? = null,
    val default: Boolean = false
)

@Serializable
data class MessageInteraction(
    val id: String,
    val type: Int,
    val name: String,
    val user: User? = null
)

@Serializable
data class MessageInteractionMetadata(
    val id: String,
    val type: Int,
    val user: User? = null,
    val authorizing_integration_owners: Map<String, String>? = null,
    val original_response_message_id: String? = null,
    val target_user: User? = null,
    val target_message_id: String? = null
)

@Serializable
data class MessageReaction(
    val emoji: Emoji,
    val count: Int,
    val count_details: ReactionCountDetails,
    val me: Boolean,
    val me_burst: Boolean,
    val burst_colors: List<String>? = null,
    val burst_count: Int? = 0
)

@Serializable
data class ReactionCountDetails(
    val burst: Int,
    val normal: Int
)

@Serializable
data class MessageReactionAdd(
    val user_id: String,
    val channel_id: String,
    val message_id: String,
    val guild_id: String? = null,
    val member: Member? = null,
    val emoji: Emoji
)

@Serializable
data class MessageReactionRemove(
    val user_id: String,
    val channel_id: String,
    val message_id: String,
    val guild_id: String? = null,
    val emoji: Emoji
)

@Serializable
data class MessageReactionRemoveAll(
    val channel_id: String,
    val message_id: String,
    val guild_id: String? = null
)

@Serializable
data class MessageReactionRemoveEmoji(
    val channel_id: String,
    val message_id: String,
    val guild_id: String? = null,
    val emoji: Emoji
)

enum class AutocompleteType {
    MENTION, USER, CHANNEL, COMMAND, EMOJI, ROLE
}

@Serializable
data class MessageAcknowledge(
    val channel_id: String,
    val message_id: String,
    val mention_count: Int? = null
)
