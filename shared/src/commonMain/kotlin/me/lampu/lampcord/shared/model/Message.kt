package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable

@Serializable
data class Message(
    val id: String,
    val channel_id: String,
    val author: User,
    val content: String,
    val timestamp: String,
    val edited_timestamp: String? = null,
    val tts: Boolean = false,
    val mention_everyone: Boolean = false,
    val attachments: List<Attachment> = emptyList(),
    val embeds: List<Embed> = emptyList(),
    val reactions: List<MessageReaction>? = null,
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
    val sendError: String? = null
)

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
    val user: User
)

@Serializable
data class MessageInteractionMetadata(
    val id: String,
    val type: Int,
    val user: User,
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
    val burst_colors: List<String>? = null
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

@Serializable
data class MessageAcknowledge(
    val channel_id: String,
    val message_id: String,
    val mention_count: Int? = null
)
