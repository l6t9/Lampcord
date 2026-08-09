package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Role
import kotlinx.serialization.json.*

class GuildStore(
    private val discordClient: DiscordClient,
    private val errorStore: AppErrorStore,
    private val scope: CoroutineScope
) {
    val guilds = mutableStateListOf<Guild>()
    val channels = mutableStateListOf<Channel>()
    val privateChannels = mutableStateListOf<Channel>()
    val forumThreads = mutableStateListOf<Channel>()
    
    var selectedGuild by mutableStateOf<Guild?>(null)
    var selectedChannel by mutableStateOf<Channel?>(null)
    var selectedThread by mutableStateOf<Channel?>(null)

    val allGuildChannels = mutableStateMapOf<String, List<Channel>>()

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

    fun markCategoryAsRead(categoryId: String, guildId: String?) {
        // Implementation
    }

    fun updateGuild(guildId: String, partial: Guild.Partial) {
        scope.launch {
            if (discordClient.updateGuild(guildId, partial)) {
                guilds.find { it.id == guildId }?.let { g ->
                    val updated = g.merge(partial)
                    val index = guilds.indexOf(g)
                    if (index != -1) guilds[index] = updated
                    if (selectedGuild?.id == guildId) selectedGuild = updated
                }
            }
        }
    }


    fun setGuilds(newGuilds: List<me.lampu.lampcord.shared.model.Guild>, order: List<String>) {
        println("GuildStore received ${newGuilds.size} guilds")
        newGuilds.forEach { rawGuild ->
            val newGuild = rawGuild.copy(emojis = rawGuild.emojis.map { it.copy(guild_id = rawGuild.id) })
            val index = guilds.indexOfFirst { it.id == newGuild.id }
            if (index != -1) {
                val existing = guilds[index]
                // Merge logic: Replicate 126.21 StoreGuilds.handleGuild
                guilds[index] = existing.copy(
                    name = newGuild.name ?: existing.name,
                    icon = newGuild.icon ?: existing.icon,
                    banner = newGuild.banner ?: existing.banner,
                    roles = if (newGuild.roles.isNotEmpty()) newGuild.roles else existing.roles,
                    features = newGuild.features ?: existing.features,
                    owner_id = newGuild.owner_id ?: existing.owner_id,
                    unavailable = newGuild.unavailable ?: existing.unavailable,
                    channels = newGuild.channels ?: existing.channels,
                    members = if (newGuild.members != null) newGuild.members else existing.members,
                    emojis = newGuild.emojis
                )
            } else {
                guilds.add(newGuild)
            }
            
            // Sync initial channels to allGuildChannels cache
            newGuild.channels?.let { guildChannels ->
                allGuildChannels[newGuild.id] = guildChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
            }
        }
        
        // Re-sort based on user_settings order (matches StoreGuildsSorted in 126.21)
        if (order.isNotEmpty()) {
            val sorted = guilds.sortedBy { guild -> 
                val pos = order.indexOf(guild.id)
                if (pos == -1) Int.MAX_VALUE else pos 
            }.toList()
            guilds.clear()
            guilds.addAll(sorted)
        }
    }

    fun handleGuildCreate(guild: me.lampu.lampcord.shared.model.Guild, order: List<String>) {
        println("GuildStore received GUILD_CREATE for ${guild.id}")
        val processedGuild = guild.copy(emojis = guild.emojis.map { it.copy(guild_id = guild.id) })
        val existingIndex = guilds.indexOfFirst { it.id == processedGuild.id }
        if (existingIndex != -1) {
            val existing = guilds[existingIndex]
            guilds[existingIndex] = existing.copy(
                name = processedGuild.name ?: existing.name,
                icon = processedGuild.icon ?: existing.icon,
                banner = processedGuild.banner ?: existing.banner,
                roles = if (processedGuild.roles.isNotEmpty()) processedGuild.roles else existing.roles,
                features = processedGuild.features ?: existing.features,
                owner_id = processedGuild.owner_id ?: existing.owner_id,
                unavailable = processedGuild.unavailable ?: existing.unavailable,
                channels = processedGuild.channels ?: existing.channels,
                members = processedGuild.members ?: existing.members,
                emojis = processedGuild.emojis
            )
            if (selectedGuild?.id == processedGuild.id) {
                selectedGuild = guilds[existingIndex]
            }
        } else {
            guilds.add(processedGuild)
        }
        
        // Sync channels to cache
        processedGuild.channels?.let { guildChannels ->
            allGuildChannels[processedGuild.id] = guildChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
            if (selectedGuild?.id == processedGuild.id) {
                channels.clear()
                channels.addAll(allGuildChannels[processedGuild.id] ?: emptyList())
            }
        }
        
        // Re-sort
        if (order.isNotEmpty()) {
            val sorted = guilds.sortedBy { g -> 
                val pos = order.indexOf(g.id)
                if (pos == -1) Int.MAX_VALUE else pos 
            }
            guilds.clear()
            guilds.addAll(sorted)
        }
    }

    fun handleGuildDelete(guildId: String) {
        println("GuildStore received GUILD_DELETE for $guildId")
        guilds.removeAll { it.id == guildId }
        allGuildChannels.remove(guildId)
        if (selectedGuild?.id == guildId) {
            selectedGuild = null
            selectedChannel = null
        }
    }

    fun handleChannelCreateOrUpdate(channel: Channel) {
        val guildId = channel.guild_id ?: return
        val currentChannels = allGuildChannels[guildId]?.toMutableList() ?: mutableListOf()
        val index = currentChannels.indexOfFirst { it.id == channel.id }
        if (index != -1) {
            currentChannels[index] = channel
        } else {
            currentChannels.add(channel)
        }
        val sorted = currentChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
        allGuildChannels[guildId] = sorted
        
        if (selectedGuild?.id == guildId) {
            channels.clear()
            channels.addAll(sorted)
        }
    }

    fun handleChannelDelete(channel: Channel) {
        val guildId = channel.guild_id ?: return
        val currentChannels = allGuildChannels[guildId]?.toMutableList() ?: return
        currentChannels.removeAll { it.id == channel.id }
        allGuildChannels[guildId] = currentChannels
        
        if (selectedGuild?.id == guildId) {
            channels.clear()
            channels.addAll(currentChannels)
            if (selectedChannel?.id == channel.id) {
                selectedChannel = null
            }
        }
    }

    fun handleRoleCreateOrUpdate(guildId: String, role: Role) {
        val guildIndex = guilds.indexOfFirst { it.id == guildId }
        if (guildIndex != -1) {
            val guild = guilds[guildIndex]
            val currentRoles = guild.roles.toMutableList()
            val roleIndex = currentRoles.indexOfFirst { it.id == role.id }
            if (roleIndex != -1) {
                currentRoles[roleIndex] = role
            } else {
                currentRoles.add(role)
            }
            guilds[guildIndex] = guild.copy(roles = currentRoles)
            if (selectedGuild?.id == guildId) {
                selectedGuild = guilds[guildIndex]
            }
        }
    }

    fun handleRoleDelete(guildId: String, roleId: String) {
        val guildIndex = guilds.indexOfFirst { it.id == guildId }
        if (guildIndex != -1) {
            val guild = guilds[guildIndex]
            val currentRoles = guild.roles.toMutableList()
            currentRoles.removeAll { it.id == roleId }
            guilds[guildIndex] = guild.copy(roles = currentRoles)
            if (selectedGuild?.id == guildId) {
                selectedGuild = guilds[guildIndex]
            }
        }
    }

    fun setPrivateChannels(newChannels: List<Channel>) {
        println("GuildStore received ${newChannels.size} private channels")
        privateChannels.clear()
        privateChannels.addAll(newChannels)
    }

    fun upsertForumThread(thread: Channel) {
        val index = forumThreads.indexOfFirst { it.id == thread.id }
        if (index != -1) {
            forumThreads[index] = thread
        } else {
            forumThreads.add(thread)
        }
        val sorted = forumThreads.sortedByDescending { it.forumSortKey() }
        forumThreads.clear()
        forumThreads.addAll(sorted)
    }

    private fun Channel.forumSortKey(): Long = last_message_id?.jsonPrimitive?.contentOrNull?.toLongOrNull() ?: id.toLong()

    fun clear() {
        guilds.clear()
        channels.clear()
        privateChannels.clear()
        forumThreads.clear()
        allGuildChannels.clear()
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
    }
}
