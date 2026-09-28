package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.Logging
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlin.time.Clock
import kotlin.time.Duration

class GuildStore(
    private val guildApi: GuildApi,
    private val errorStore: AppErrorStore,
    private val selectionStore: SelectionStore,
    private val entityStore: EntityStore,
    private val userGuildSettingsStore: UserGuildSettingsStore,
    private val readStateStore: ReadStateStore,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) {
    private val _guildIds = MutableStateFlow<List<String>>(emptyList())
    val guilds: StateFlow<List<Guild>> = combine(_guildIds, entityStore.guilds) { ids, allGuilds ->
        ids.mapNotNull { allGuilds[it] }
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    private val _privateChannelIds = MutableStateFlow<Set<String>>(emptySet())
    val privateChannels: StateFlow<List<Channel>> = combine(_privateChannelIds, entityStore.channels) { ids, allChannels ->
        ids.mapNotNull { allChannels[it] }
            .sortedByDescending { it.lastMessageId()?.toLongOrNull() ?: 0L }
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    val allGuildChannels = entityStore.channels

    fun setGuilds(newGuilds: List<Guild>, order: List<String>) {
        Logging.i("GuildStore", "Setting ${newGuilds.size} guilds")
        newGuilds.forEach { entityStore.updateGuild(it) }
        val sortedIds = if (order.isNotEmpty()) {
            val orderIndex = order.withIndex().associate { it.value to it.index }
            newGuilds.map { it.id }.sortedWith(compareBy({ orderIndex[it] ?: -1 }, { it }))
        } else {
            newGuilds.sortedWith(Comparator { a, b ->
                val mutedA = userGuildSettingsStore.isGuildMuted(a.id)
                val mutedB = userGuildSettingsStore.isGuildMuted(b.id)
                if (mutedA != mutedB) return@Comparator if (mutedA) 1 else -1

                val j1 = userStore.getCurrentMember(a.id)?.joined_at ?: ""
                val j2 = userStore.getCurrentMember(b.id)?.joined_at ?: ""

                if (j1.isBlank() && j2.isNotBlank()) return@Comparator -1
                if (j1.isNotBlank() && j2.isBlank()) return@Comparator 1
                val joinedCompare = j1.compareTo(j2)
                if (joinedCompare != 0) return@Comparator joinedCompare

                val n1 = a.name ?: ""
                val n2 = b.name ?: ""
                n1.compareTo(n2, ignoreCase = true)
            }).map { it.id }
        }
        _guildIds.value = sortedIds
    }

    fun handleGuildCreate(guild: Guild, order: List<String>) {
        Logging.i("GuildStore", "Handling GUILD_CREATE: ${guild.name} (${guild.id})")
        entityStore.updateGuild(guild)
        if (guild.id !in _guildIds.value) {
            val newList = _guildIds.value + guild.id
            _guildIds.value = if (order.isNotEmpty()) {
                val orderIndex = order.withIndex().associate { it.value to it.index }
                newList.sortedWith(compareBy({ orderIndex[it] ?: -1 }, { it }))
            } else {
                val allGuilds = newList.mapNotNull { id -> entityStore.guilds.value[id] }
                allGuilds.sortedWith(Comparator { a, b ->
                    val mutedA = userGuildSettingsStore.isGuildMuted(a.id)
                    val mutedB = userGuildSettingsStore.isGuildMuted(b.id)
                    if (mutedA != mutedB) return@Comparator if (mutedA) 1 else -1

                    val j1 = userStore.getCurrentMember(a.id)?.joined_at ?: ""
                    val j2 = userStore.getCurrentMember(b.id)?.joined_at ?: ""

                    if (j1.isBlank() && j2.isNotBlank()) return@Comparator -1
                    if (j1.isNotBlank() && j2.isBlank()) return@Comparator 1
                    val joinedCompare = j1.compareTo(j2)
                    if (joinedCompare != 0) return@Comparator joinedCompare

                    val n1 = a.name ?: ""
                    val n2 = b.name ?: ""
                    n1.compareTo(n2, ignoreCase = true)
                }).map { it.id }
            }
        }
    }

    fun handleGuildDelete(guildId: String) {
        Logging.i("GuildStore", "Handling GUILD_DELETE: $guildId")
        _guildIds.value -= guildId
        entityStore.removeGuild(guildId)
    }

    fun reorderGuilds(order: List<String>) {
        if (order.isEmpty()) return
        val currentIds = _guildIds.value
        val orderIndex = order.withIndex().associate { it.value to it.index }
        val sortedIds = currentIds.sortedWith(compareBy({ orderIndex[it] ?: -1 }, { it }))
        if (sortedIds != currentIds) {
            _guildIds.value = sortedIds
        }
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
            if (guildApi.updateGuild(guildId, partial)) {
                entityStore.guilds.value[guildId]?.let { g ->
                    entityStore.updateGuild(g.merge(partial))
                }
            }
        }
    }

    fun markGuildAsRead(guildId: String) {
        markGuildsAsRead(listOf(guildId), "guild")
    }

    fun markFolderAsRead(folder: GuildFolder) {
        val guildIds = folder.guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull }.distinct()
        markGuildsAsRead(guildIds, "folder")
    }

    private fun markGuildsAsRead(guildIds: Collection<String>, target: String) {
        if (guildIds.isEmpty()) return
        scope.launch {
            try {
                val readStates = allGuildChannels.value.values
                    .asSequence()
                    .filter {
                        it.guild_id in guildIds &&
                            (readStateStore.isUnread(it) || readStateStore.getMentionCount(it.id) > 0)
                    }
                    .mapNotNull { channel ->
                        channel.lastMessageId()
                            ?.takeIf { it.toLongOrNull()?.let { id -> id > 0L } == true }
                            ?.let { channel.id to it }
                    }
                    .toMap()

                if (!readStateStore.ackBulk(readStates)) {
                    errorStore.pushError("Failed to mark $target as read.")
                }
            } catch (e: Exception) {
                errorStore.pushError("Error marking $target as read: ${e.message}")
            }
        }
    }

    fun leaveGuild(guildId: String, onLeave: () -> Unit) {
        scope.launch {
            try {
                if (guildApi.leaveGuild(guildId)) {
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
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(muted = false, mute_config = null))
        }
    }

    fun muteGuild(guildId: String, duration: Duration?) {
        val muteConfig = if (duration != null) {
            val endTime = (Clock.System.now() + duration).toString()
            MuteConfig(end_time = endTime)
        } else MuteConfig(end_time = null)

        scope.launch {
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(muted = true, mute_config = muteConfig))
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
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(channel_overrides = currentOverrides))
        }
    }

    fun muteChannelForDuration(guildId: String, channelId: String, duration: Duration?) {
        val effectiveGuildId = if (guildId == "@me") null else guildId
        val guildSettings = userGuildSettingsStore.userGuildSettings.value[effectiveGuildId]
        val currentOverrides = guildSettings?.channel_overrides?.toMutableList() ?: mutableListOf()
        val index = currentOverrides.indexOfFirst { it.channel_id == channelId }
        val muteConfig = if (duration != null) {
            MuteConfig(end_time = (Clock.System.now() + duration).toString())
        } else null

        val newOverride = if (index != -1) {
            currentOverrides[index].copy(muted = true, mute_config = muteConfig)
        } else {
            ChannelOverride(channel_id = channelId, muted = true, mute_config = muteConfig)
        }
        if (index != -1) currentOverrides[index] = newOverride else currentOverrides.add(newOverride)

        scope.launch {
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(channel_overrides = currentOverrides))
        }
    }

    fun unmuteChannel(guildId: String, channelId: String) {
        val effectiveGuildId = if (guildId == "@me") null else guildId
        val guildSettings = userGuildSettingsStore.userGuildSettings.value[effectiveGuildId] ?: return
        val currentOverrides = guildSettings.channel_overrides.toMutableList()
        val index = currentOverrides.indexOfFirst { it.channel_id == channelId }
        if (index == -1) return

        currentOverrides[index] = currentOverrides[index].copy(muted = false, mute_config = null)

        scope.launch {
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(channel_overrides = currentOverrides))
        }
    }

    fun setChannelNotificationMode(guildId: String, channelId: String, mode: Int) {
        val effectiveGuildId = if (guildId == "@me") null else guildId
        val guildSettings = userGuildSettingsStore.userGuildSettings.value[effectiveGuildId]
        val currentOverrides = guildSettings?.channel_overrides?.toMutableList() ?: mutableListOf()
        val index = currentOverrides.indexOfFirst { it.channel_id == channelId }

        val newOverride = if (index != -1) {
            currentOverrides[index].copy(message_notifications = mode)
        } else {
            ChannelOverride(channel_id = channelId, message_notifications = mode)
        }
        if (index != -1) currentOverrides[index] = newOverride else currentOverrides.add(newOverride)

        scope.launch {
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(channel_overrides = currentOverrides))
        }
    }

    fun setServerDMsAllowed(guildId: String, allowed: Boolean) {
        scope.launch {
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(message_notifications = if (allowed) 0 else 2))
        }
    }

    fun setHideMutedChannels(guildId: String, hide: Boolean) {
        scope.launch {
            guildApi.updateUserGuildSettings(guildId, UserGuildSettings.Partial(hide_muted_channels = hide))
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
        return allGuildChannels.value.values.any { 
            it.guild_id == guildId && !userGuildSettingsStore.isChannelMuted(it.guild_id, it.id) && readStateStore.isUnread(it) 
        }
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
