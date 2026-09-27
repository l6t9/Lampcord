package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.UserGuildSettings
import me.lampu.lampcord.shared.notifications.IncomingNotificationData
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.state.GatewayEventHandler
import me.lampu.lampcord.shared.state.MessageStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.UserGuildSettingsStore
import me.lampu.lampcord.shared.state.UserStore

class NotificationEventHandler(
    private val json: Json,
    private val userStore: UserStore,
    private val messageStore: MessageStore,
    private val navigationStore: NavigationStore,
    private val userGuildSettingsStore: UserGuildSettingsStore,
    private val relationshipStore: RelationshipStore,
    private val entityStore: EntityStore,
    private val notifier: MessageNotifier?
) : GatewayEventHandler {
    override val supportedEvents = setOf("MESSAGE_CREATE", "USER_GUILD_SETTINGS_UPDATE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "MESSAGE_CREATE" -> if (notifier != null) handleMessageCreate(data)
            "USER_GUILD_SETTINGS_UPDATE" -> handleUserGuildSettingsUpdate(data)
        }
    }

    private fun handleUserGuildSettingsUpdate(data: JsonElement) {
        try {
            val partial = json.decodeFromJsonElement<UserGuildSettings.Partial>(data)
            userGuildSettingsStore.handlePartialUpdate(partial)
        } catch (_: Exception) {
        }
    }

    private fun handleMessageCreate(data: JsonElement) {
        val notifier = notifier ?: return
        val message = try {
            json.decodeFromJsonElement<Message>(data)
        } catch (_: Exception) {
            return
        }

        val author = message.author ?: return
        val currentUser = userStore.currentUser.value ?: return

        if (author.id == currentUser.id) return
        if (message.type != null && message.type != 0 && message.type != 19) return
        if (message.content.isBlank() && message.attachments.isEmpty() && message.embeds.isEmpty() && message.sticker_items.isNullOrEmpty()) return
        if (navigationStore.selectedChannel?.id == message.channel_id) return
        if (navigationStore.selectedThread?.id == message.channel_id) return
        if (relationshipStore.relationships.value.any { it.type == 2 && (it.user?.id == author.id || it.user_id == author.id) }) return

        val guildId = message.guild_id
        val isDm = guildId == null

        val currentMember = guildId?.let { userStore.getMember(it, currentUser.id) }
        val isDirectMention = messageStore.isMessageMentioningMe(message, currentUser, currentMember)

        if (isDm) {
            val meSettings = userGuildSettingsStore.userGuildSettings.value["@me"]
            val dmOverride = meSettings?.channel_overrides?.find { it.channel_id == message.channel_id }
            if (dmOverride?.muted == true) {
                val muteEndTime = dmOverride.mute_config?.end_time
                val isMuteActive = if (muteEndTime != null) {
                    try { kotlin.time.Instant.parse(muteEndTime) > kotlin.time.Clock.System.now() } catch (_: Exception) { true }
                } else true
                if (isMuteActive) return
            }

            dispatchNotification(message, author, isDm = true, isMention = isDirectMention)
            return
        }

        val guildSettings = userGuildSettingsStore.userGuildSettings.value[guildId]
        val channelOverride = guildSettings?.channel_overrides?.find { it.channel_id == message.channel_id }

        val guildMuted = guildSettings?.muted == true && (guildSettings.mute_config?.end_time?.let {
            try { kotlin.time.Instant.parse(it) > kotlin.time.Clock.System.now() } catch (_: Exception) { true }
        } ?: true)

        val channelMuted = channelOverride?.muted == true && (channelOverride.mute_config?.end_time?.let {
            try { kotlin.time.Instant.parse(it) > kotlin.time.Clock.System.now() } catch (_: Exception) { true }
        } ?: true)

        if ((guildMuted || channelMuted) && !isDirectMention) return

        val channelLevel = channelOverride?.message_notifications
        val effectiveNotifyLevel = when (channelLevel) {
            NTF_CHANNEL_ALL -> NTF_ALL
            NTF_CHANNEL_MENTIONS_ONLY -> NTF_MENTIONS_ONLY
            NTF_CHANNEL_NONE -> NTF_NONE
            else -> guildSettings?.message_notifications ?: NTF_MENTIONS_ONLY
        }

        val isEveryoneMentionAllowed = message.mention_everyone == true && guildSettings?.suppress_everyone != true
        val isRoleMentionAllowed = message.mention_roles.isNotEmpty() && guildSettings?.suppress_roles != true

        val shouldNotify = when {
            isDirectMention -> true
            isEveryoneMentionAllowed || isRoleMentionAllowed -> effectiveNotifyLevel != NTF_NONE
            else -> effectiveNotifyLevel == NTF_ALL
        }

        if (!shouldNotify) return

        dispatchNotification(message, author, isDm = false, isMention = isDirectMention || isEveryoneMentionAllowed || isRoleMentionAllowed)
    }

    private fun dispatchNotification(message: Message, author: me.lampu.lampcord.shared.model.User, isDm: Boolean, isMention: Boolean) {
        val notifier = notifier ?: return
        val channel = entityStore.channels.value[message.channel_id]
        val guild = message.guild_id?.let { entityStore.guilds.value[it] }

        val displayName = message.member?.nick
            ?: author.global_name
            ?: author.username
            ?: "Unknown"

        val channelLabel = when {
            !isDm -> channel?.name?.let { "#$it" }
            channel?.type == 3 -> channel.name
                ?: channel.recipients?.mapNotNull { it.global_name ?: it.username }?.joinToString(", ")
            else -> null
        }

        notifier.showMessageNotification(
            IncomingNotificationData(
                message = message,
                channel = channel,
                guild = guild,
                authorDisplayName = displayName,
                authorAvatarUrl = CdnUrls.getUserAvatarUrl(author.id, author.avatar, 128),
                channelLabel = channelLabel,
                isDm = isDm,
                isMention = isMention
            )
        )
    }

    companion object {
        private const val NTF_ALL = 0
        private const val NTF_MENTIONS_ONLY = 1
        private const val NTF_NONE = 2
        private const val NTF_CHANNEL_ALL = 0
        private const val NTF_CHANNEL_MENTIONS_ONLY = 1
        private const val NTF_CHANNEL_NONE = 2
    }
}
