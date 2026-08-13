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
        } catch (e: Exception) {
        }
    }

    private fun handleMessageCreate(data: JsonElement) {
        val notifier = notifier ?: return
        val message = try {
            json.decodeFromJsonElement<Message>(data)
        } catch (e: Exception) {
            return
        }

        val author = message.author ?: return
        val currentUser = userStore.currentUser.value ?: return

        if (author.id == currentUser.id) return
        if (notifier.isInForeground && navigationStore.selectedChannel?.id == message.channel_id) return
        if (notifier.isInForeground && navigationStore.selectedThread?.id == message.channel_id) return
        if (message.type != null && message.type != 0 && message.type != 19) return
        if (message.content.isBlank() && message.attachments.isEmpty() && message.embeds.isEmpty() && message.sticker_items.isNullOrEmpty()) return

        if (relationshipStore.relationships.value.any { it.type == 2 && (it.user?.id == author.id || it.user_id == author.id) }) return

        val settingsGuildId = message.guild_id ?: "@me"
        val guildSettings = userGuildSettingsStore.userGuildSettings.value[settingsGuildId]
        val channelOverride = guildSettings?.channel_overrides?.find { it.channel_id == message.channel_id }

        if (guildSettings != null) {
            val channelMuted = channelOverride?.muted == true && (channelOverride.mute_config?.end_time?.let {
                try { kotlin.time.Instant.parse(it) > kotlin.time.Clock.System.now() } catch (_: Exception) { false }
            } ?: true)
            val guildMuted = guildSettings.muted && (guildSettings.mute_config?.end_time?.let {
                try { kotlin.time.Instant.parse(it) > kotlin.time.Clock.System.now() } catch (_: Exception) { false }
            } ?: true)
            if (guildMuted || channelMuted) return
            if (guildSettings.mobile_push == false) return
        }

        val currentMember = message.guild_id?.let { userStore.getMember(it, currentUser.id) }
        val isMention = messageStore.isMessageMentioningMe(message, currentUser, currentMember)

        if (guildSettings != null && !isMention) {
            val overrideLevel = channelOverride?.message_notifications
            val notifyLevel = when (overrideLevel) {
                NTF_CHANNEL_ALL -> NTF_ALL
                NTF_CHANNEL_MENTIONS_ONLY -> NTF_MENTIONS_ONLY
                else -> guildSettings.message_notifications
            }
            if (notifyLevel == NTF_NONE || notifyLevel == NTF_MENTIONS_ONLY) return
        }

        val channel = entityStore.channels.value[message.channel_id]
        val guild = message.guild_id?.let { entityStore.guilds.value[it] }
        val isDm = message.guild_id == null

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
        private const val NTF_CHANNEL_ALL = 1
        private const val NTF_CHANNEL_MENTIONS_ONLY = 2
    }
}
