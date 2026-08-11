package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.*
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlin.time.Clock
import kotlin.time.Duration

class GuildStore(
    private val discordClient: DiscordClient,
    private val errorStore: AppErrorStore,
    private val selectionStore: SelectionStore,
    private val entityStore: EntityStore,
    private val userGuildSettingsStore: UserGuildSettingsStore,
    private val readStateStore: ReadStateStore,
    private val scope: CoroutineScope
) {
    private val _guildIds = MutableStateFlow<List<String>>(emptyList())
    val guilds: StateFlow<List<Guild>> = combine(_guildIds, entityStore.guilds) { ids, allGuilds ->
        ids.mapNotNull { allGuilds[it] }
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    private val _privateChannelIds = MutableStateFlow<Set<String>>(emptySet())
    val privateChannels: StateFlow<List<Channel>> = combine(_privateChannelIds, entityStore.channels) { ids, allChannels ->
        ids.mapNotNull { allChannels[it] }
            .sortedByDescending { it.lastMessageId() ?: "" }
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    val allGuildChannels = entityStore.channels

    fun setGuilds(newGuilds: List<Guild>, order: List<String>) {
        newGuilds.forEach { entityStore.updateGuild(it) }
        
        val sortedIds = if (order.isNotEmpty()) {
            newGuilds.map { it.id }.sortedBy { id ->
                val pos = order.indexOf(id)
                if (pos == -1) Int.MAX_VALUE else pos
            }
        } else {
            newGuilds.map { it.id }
        }
        _guildIds.value = sortedIds
    }

    fun handleGuildCreate(guild: Guild, order: List<String>) {
        entityStore.updateGuild(guild)
        if (guild.id !in _guildIds.value) {
            val newList = _guildIds.value + guild.id
            _guildIds.value = if (order.isNotEmpty()) {
                newList.sortedBy { id ->
                    val pos = order.indexOf(id)
                    if (pos == -1) Int.MAX_VALUE else pos
                }
            } else newList
        }
    }

    fun handleGuildDelete(guildId: String) {
        _guildIds.value = _guildIds.value - guildId
        entityStore.removeGuild(guildId)
    }

    fun setPrivateChannels(channels: List<Channel>) {
        channels.forEach { entityStore.updateChannel(it) }
        _privateChannelIds.value = channels.map { it.id }.toSet()
    }

    fun handleChannelCreateOrUpdate(channel: Channel) {
        entityStore.updateChannel(channel)
        val updated = entityStore.channels.value[channel.id] ?: channel
        if (updated.guild_id == null && (updated.type == 1 || updated.type == 3)) {
            _privateChannelIds.update { it + updated.id }
        }
    }

    fun handleChannelDelete(channel: Channel) {
        _privateChannelIds.update { it - channel.id }
        entityStore.removeChannel(channel.id)
    }

    fun updateGuild(guildId: String, partial: Guild.Partial) {
        scope.launch {
            if (discordClient.updateGuild(guildId, partial)) {
                entityStore.guilds.value[guildId]?.let { g ->
                    entityStore.updateGuild(g.merge(partial))
                }
            }
        }
    }

    fun markGuildAsRead(guildId: String) {
        scope.launch {
            try {
                if (!discordClient.ackBulk(listOf(guildId))) {
                    errorStore.pushError("Failed to mark guild as read.")
                }
            } catch (e: Exception) {
                errorStore.pushError("Error marking guild as read: ${e.message}")
            }
        }
    }

    fun leaveGuild(guildId: String, onLeave: () -> Unit) {
        scope.launch {
            try {
                if (discordClient.leaveGuild(guildId)) {
                    handleGuildDelete(guildId)
                    onLeave()
                } else {
                    errorStore.pushError("Failed to leave guild.")
                }
            } catch (e: Exception) {
                errorStore.pushError("Error leaving guild: ${e.message}")
            }
        }
    }

    fun toggleMuteGuild(guildId: String) {
        if (userGuildSettingsStore.isGuildMuted(guildId)) unmuteGuild(guildId)
        else muteGuild(guildId, null)
    }

    fun unmuteGuild(guildId: String) {
        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(muted = false, mute_config = null))
        }
    }

    fun muteGuild(guildId: String, duration: Duration?) {
        val muteConfig = if (duration != null) {
            val endTime = (Clock.System.now() + duration).toString()
            MuteConfig(end_time = endTime)
        } else MuteConfig(end_time = null)

        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(muted = true, mute_config = muteConfig))
        }
    }

    fun toggleMuteChannel(guildId: String, channelId: String) {
        val effectiveGuildId = if (guildId == "@me") null else guildId
        val guildSettings = userGuildSettingsStore.userGuildSettings.value[effectiveGuildId] ?: return
        val currentOverrides = guildSettings.channel_overrides.toMutableList()
        val index = currentOverrides.indexOfFirst { it.channel_id == channelId }
        
        val newOverride = if (index != -1) {
            currentOverrides[index].copy(muted = !currentOverrides[index].muted)
        } else {
            ChannelOverride(channel_id = channelId, muted = true)
        }
        
        if (index != -1) currentOverrides[index] = newOverride else currentOverrides.add(newOverride)

        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(channel_overrides = currentOverrides))
        }
    }

    fun setServerDMsAllowed(guildId: String, allowed: Boolean) {
        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(message_notifications = if (allowed) 0 else 2))
        }
    }

    fun setHideMutedChannels(guildId: String, hide: Boolean) {
        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(hide_muted_channels = hide))
        }
    }

    fun markCategoryAsRead(categoryId: String) {
        val channel = allGuildChannels.value[categoryId] ?: return
        val guildId = channel.guild_id ?: return
        markGuildAsRead(guildId)
    }

    fun isFolderUnread(folder: GuildFolder): Boolean {
        return folder.guild_ids.any { el ->
            val id = el.jsonPrimitive.contentOrNull ?: return@any false
            isGuildUnread(id)
        }
    }

    fun getFolderMentionCount(folder: GuildFolder): Int {
        return folder.guild_ids.sumOf { el ->
            val id = el.jsonPrimitive.contentOrNull ?: return@sumOf 0
            getGuildMentionCount(id)
        }
    }

    fun isGuildUnread(guildId: String): Boolean {
        if (userGuildSettingsStore.isGuildMuted(guildId)) return false
        return allGuildChannels.value.values.filter { it.guild_id == guildId }.any { readStateStore.isUnread(it) }
    }

    fun getGuildMentionCount(guildId: String): Int {
        return allGuildChannels.value.values.filter { it.guild_id == guildId }.sumOf { readStateStore.getMentionCount(it.id) }
    }

    fun getForumThreads(channelId: String): List<Channel> {
        return allGuildChannels.value.values.filter { it.parent_id == channelId }
    }

    fun clear() {
        _guildIds.value = emptyList()
        _privateChannelIds.value = emptySet()
    }
}
