package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.utils.DiscordUrl
import me.lampu.lampcord.shared.utils.Logging

class DiscordLinkHandler(
    private val channelNavigator: ChannelNavigator,
    private val navigationStore: NavigationStore,
    private val entityStore: EntityStore,
    private val messageStore: MessageStore,
    private val profileStore: ProfileStore,
    private val channelApi: ChannelApi,
    private val messageApi: MessageApi,
    private val guildApi: GuildApi,
    private val scope: CoroutineScope
) {
    var externalLinkOpener: (String) -> Unit = { url ->
        Logging.i(TAG, "No external link opener registered for $url")
    }

    fun open(url: String): Boolean {
        val target = DiscordUrl.parse(url) ?: return false
        scope.launch { navigate(target) }
        return true
    }

    private suspend fun navigate(target: DiscordUrl.Target) {
        when (target) {
            is DiscordUrl.Target.Channel -> {
                val channelId = if (entityStore.channels.value.containsKey(target.channelId)) {
                    target.channelId
                } else {
                    withContext(Dispatchers.IO) {
                        runCatching { channelApi.getChannel(target.channelId) }.getOrNull()
                    }?.also { entityStore.updateChannel(it) }?.id ?: run {
                        Logging.w(TAG, "Could not open channel ${target.channelId}")
                        return
                    }
                }
                channelNavigator.navigateToChannel(channelId, target.guildId)
                target.messageId?.let { jumpToMessage(channelId, it) }
            }

            is DiscordUrl.Target.Thread -> {
                val parent = withContext(Dispatchers.IO) {
                    runCatching { channelApi.getChannel(target.parentId) }.getOrNull()
                } ?: run {
                    Logging.w(TAG, "Could not open thread parent ${target.parentId}")
                    return
                }
                entityStore.updateChannel(parent)
                channelNavigator.navigateToChannel(parent.id, target.guildId)
                val thread = withContext(Dispatchers.IO) {
                    runCatching { channelApi.getChannel(target.threadId) }.getOrNull()
                } ?: return
                entityStore.updateChannel(thread)
                navigationStore.selectThread(thread)
            }

            is DiscordUrl.Target.Invite -> {
                val invite = withContext(Dispatchers.IO) {
                    runCatching { guildApi.resolveInvite(target.code) }.getOrNull()
                } ?: run {
                    Logging.w(TAG, "Could not resolve the invite ${target.code}")
                    return
                }
                val guild = invite.guild
                if (guild != null && entityStore.guilds.value.containsKey(guild.id)) {
                    channelNavigator.navigateToGuild(guild.id)
                } else {
                    navigationStore.pendingInviteCode = target.code
        externalLinkOpener("https://discord.gg/${target.code}")
                }
            }

            is DiscordUrl.Target.Attachment ->
                Logging.i(TAG, "Attachment link: ${target.filename}")

            is DiscordUrl.Target.User ->
                profileStore.showProfile(target.userId, navigationStore.selectedGuild?.id)
        }
    }

    private suspend fun jumpToMessage(channelId: String, messageId: String) {
        if (entityStore.channels.value[channelId] == null) return
        if (messageStore.hasMessages(channelId) &&
            messageStore.messages.value.any { it.id == messageId }
        ) {
            messageStore.scrollToMessageId = messageId
            return
        }
        val around = withContext(Dispatchers.IO) {
            runCatching { messageApi.getMessagesAround(channelId, messageId) }.getOrNull()
        }.orEmpty()
        if (around.isNotEmpty()) {
            messageStore.addMessages(channelId, around)
        }
        messageStore.scrollToMessageId = messageId
    }

    private companion object {
        const val TAG = "links"
    }
}