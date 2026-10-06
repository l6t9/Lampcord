package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.state.*

class GuildEventHandler(
    private val json: Json,
    private val entityStore: EntityStore,
    private val guildStore: GuildStore,
    private val navigationStore: NavigationStore,
    private val settingsStore: SettingsStore,
    private val userStore: UserStore,
    private val presenceStore: PresenceStore
) : GatewayEventHandler {
    override val supportedEvents = setOf(
        "GUILD_CREATE", "GUILD_UPDATE", "GUILD_DELETE",
        "CHANNEL_CREATE", "CHANNEL_UPDATE", "CHANNEL_DELETE",
        "DM_CREATE",
        "THREAD_CREATE", "THREAD_UPDATE", "THREAD_DELETE", "THREAD_LIST_SYNC",
        "GUILD_MEMBERS_CHUNK"
    )

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "GUILD_CREATE" -> handleGuildCreate(data)
            "GUILD_MEMBERS_CHUNK" -> handleGuildMembersChunk(data)
            "GUILD_UPDATE" -> handleGuildUpdate(data)
            "GUILD_DELETE" -> handleGuildDelete(data)
            "CHANNEL_CREATE", "THREAD_CREATE", "DM_CREATE" -> handleChannelUpdate(data)
            "CHANNEL_UPDATE", "THREAD_UPDATE" -> handleChannelUpdate(data)
            "CHANNEL_DELETE", "THREAD_DELETE" -> handleChannelDelete(data)
            "THREAD_LIST_SYNC" -> handleThreadListSync(data)
        }
    }

    private fun handleGuildMembersChunk(data: JsonElement) {
        val object_ = data.jsonObject
        val guildId = object_["guild_id"]?.jsonPrimitive?.contentOrNull ?: return
        val members = object_["members"] as? JsonArray ?: return

        val pairs = ArrayList<Pair<String, Member>>(members.size)
        val raws = ArrayList<JsonObject?>(members.size)
        members.forEach { element ->
            val memberJson = element as? JsonObject ?: return@forEach
            val member = runCatching { json.decodeFromJsonElement<Member>(memberJson) }.getOrNull() ?: return@forEach
            val userId = member.user?.id
                ?: memberJson["user"]?.jsonObject?.get("id")?.jsonPrimitive?.contentOrNull
                ?: return@forEach
            pairs.add(userId to member)
            raws.add(memberJson)
        }
        if (pairs.isNotEmpty()) userStore.cacheMembers(guildId, pairs, raws)

        val presenceUpdates = (object_["presences"] as? JsonArray)?.mapNotNull { element ->
            val presence = element as? JsonObject ?: return@mapNotNull null
            runCatching { json.decodeFromJsonElement<PresenceUpdate>(presence) }.getOrNull()
        }.orEmpty()
        presenceUpdates.forEach { update ->
            presenceStore.applyPresences(listOf(update.copy(guild_id = update.guild_id ?: guildId)))
        }
    }

    private fun handleGuildCreate(data: JsonElement) {
        val guild = json.decodeFromJsonElement<Guild>(data)
        entityStore.updateGuild(guild)
        
        val settings = settingsStore.userSettings
        val guildOrder = settings?.guild_folders?.flatMap { folder ->
            folder.guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull }
        }?.takeIf { it.isNotEmpty() }
            ?: settings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() }
            ?: emptyList()
        guildStore.handleGuildCreate(guild, guildOrder)
        
        data.jsonObject["members"]?.jsonArray?.forEach { memberData ->
            val member = json.decodeFromJsonElement<Member>(memberData)
            val userId = member.userId() ?: return@forEach
            userStore.cacheMember(guild.id, userId, member, memberData.jsonObject)
        }

        guild.presences?.forEach { presence ->
            presenceStore.handlePresenceUpdate(presence.copy(guild_id = guild.id))
        }
    }

    private fun handleGuildUpdate(data: JsonElement) {
        val guild = json.decodeFromJsonElement<Guild>(data)
        entityStore.updateGuild(guild)
    }

    private fun handleGuildDelete(data: JsonElement) {
        val jsonObject = data as? JsonObject ?: return
        val id = jsonObject["id"]?.jsonPrimitive?.content ?: return
        val unavailable = jsonObject["unavailable"]?.jsonPrimitive?.booleanOrNull ?: false
        
        if (!unavailable) {
            guildStore.handleGuildDelete(id)
            entityStore.removeGuild(id)
            userStore.removeGuild(id)
            presenceStore.removeGuild(id)
        }
    }

    private fun handleChannelUpdate(data: JsonElement) {
        val channel = json.decodeFromJsonElement<Channel>(data)
        entityStore.updateChannel(channel)
        guildStore.handleChannelCreateOrUpdate(channel)
    }

    private fun handleChannelDelete(data: JsonElement) {
        val channel = json.decodeFromJsonElement<Channel>(data)
        entityStore.removeChannel(channel.id)
        guildStore.handleChannelDelete(channel)
    }

    private fun handleThreadListSync(data: JsonElement) {
        val obj = data.jsonObject
        val threads = obj["threads"]?.jsonArray?.map { json.decodeFromJsonElement<Channel>(it) } ?: emptyList()
        val guildId = obj["guild_id"]?.jsonPrimitive?.content ?: return
        
        threads.forEach { 
            val thread = it.copy(guild_id = guildId)
            entityStore.updateChannel(thread)
            guildStore.handleChannelCreateOrUpdate(thread)
        }
    }
}
