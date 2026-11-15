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
        "THREAD_MEMBER_UPDATE", "THREAD_MEMBERS_UPDATE",
        "GUILD_MEMBERS_CHUNK"
    )

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "GUILD_CREATE" -> handleGuildCreate(data)
            "GUILD_MEMBERS_CHUNK" -> handleGuildMembersChunk(data)
            "GUILD_UPDATE" -> handleGuildUpdate(data)
            "GUILD_DELETE" -> handleGuildDelete(data)
            "CHANNEL_CREATE", "THREAD_CREATE", "DM_CREATE" -> handleChannelUpdate(type, data)
            "CHANNEL_UPDATE", "THREAD_UPDATE" -> handleChannelUpdate(type, data)
            "CHANNEL_DELETE", "THREAD_DELETE" -> handleChannelDelete(type, data)
            "THREAD_LIST_SYNC" -> handleThreadListSync(data)
            "THREAD_MEMBER_UPDATE" -> handleThreadMemberUpdate(data)
            "THREAD_MEMBERS_UPDATE" -> handleThreadMembersUpdate(data)
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
        val obj = data.jsonObject
        val guild = json.decodeFromJsonElement<Guild>(data)
        entityStore.updateGuild(guild)

        val settings = settingsStore.userSettings
        val guildOrder = settings?.guild_folders?.flatMap { folder ->
            folder.guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull }
        }?.takeIf { it.isNotEmpty() }
            ?: settings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() }
            ?: emptyList()
        guildStore.handleGuildCreate(guild, guildOrder)

        obj["members"]?.jsonArray?.forEach { memberData ->
            val member = json.decodeFromJsonElement<Member>(memberData)
            val userId = member.userId() ?: return@forEach
            userStore.cacheMember(guild.id, userId, member, memberData.jsonObject)
        }

        guild.presences?.forEach { presence ->
            presenceStore.handlePresenceUpdate(presence.copy(guild_id = guild.id))
        }

        val joined = mutableListOf<String>()
        obj["threads"]?.jsonArray?.forEach { threadEl ->
            val threadObj = threadEl.jsonObject
            val thread = runCatching { json.decodeFromJsonElement<Channel>(threadEl) }.getOrNull() ?: return@forEach
            val withGuild = thread.copy(guild_id = thread.guild_id ?: guild.id)
            entityStore.updateChannel(withGuild)
            guildStore.handleChannelCreateOrUpdate(withGuild)
            if (threadObj["member"] != null) joined.add(withGuild.id)
        }
        if (joined.isNotEmpty()) guildStore.setJoinedThreads(joined)
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

    private fun handleChannelUpdate(type: String, data: JsonElement) {
        val channel = json.decodeFromJsonElement<Channel>(data)
        entityStore.updateChannel(channel)
        guildStore.handleChannelCreateOrUpdate(channel)

        if (type == "THREAD_CREATE") {
            val obj = data.jsonObject
            val isThread = channel.type in listOf(10, 11, 12)
            if (isThread) {
                // If gateway provides current user's thread member, or thread is newly created by us, treat as joined.
                val hasMember = obj["member"] != null
                val newlyCreated = obj["newly_created"]?.jsonPrimitive?.booleanOrNull == true
                val ownerIsMe = channel.owner_id != null && channel.owner_id == userStore.currentUser.value?.id
                if (hasMember || (newlyCreated && ownerIsMe)) {
                    guildStore.setJoinedThreads(listOf(channel.id))
                }
            }
        }
    }

    private fun handleChannelDelete(type: String, data: JsonElement) {
        val channel = json.decodeFromJsonElement<Channel>(data)
        entityStore.removeChannel(channel.id)
        guildStore.handleChannelDelete(channel)
        if (type == "THREAD_DELETE" || channel.type in listOf(10, 11, 12)) {
            guildStore.removeJoinedThreads(listOf(channel.id))
        }
    }

    private fun handleThreadListSync(data: JsonElement) {
        val obj = data.jsonObject
        val threads = obj["threads"]?.jsonArray?.mapNotNull {
            runCatching { json.decodeFromJsonElement<Channel>(it) }.getOrNull()
        } ?: emptyList()
        val guildId = obj["guild_id"]?.jsonPrimitive?.content ?: return

        threads.forEach {
            val thread = it.copy(guild_id = guildId)
            entityStore.updateChannel(thread)
            guildStore.handleChannelCreateOrUpdate(thread)
        }

        val channelIds = obj["channel_ids"]?.jsonArray?.mapNotNull { it.jsonPrimitive.contentOrNull }
        val joinedThreadIds = obj["members"]?.jsonArray?.mapNotNull { memberEl ->
            val m = memberEl.jsonObject
            m["id"]?.jsonPrimitive?.contentOrNull
                ?: m["user_id"]?.jsonPrimitive?.contentOrNull?.let { _ ->
                    // some payload shapes vary; prefer thread id
                    m["id"]?.jsonPrimitive?.contentOrNull
                }
        } ?: emptyList()

        // Replace joined membership scope for synced parent channels when provided; otherwise additively update.
        val current = guildStore.joinedThreadIds.value
        if (channelIds.isNullOrEmpty()) {
            if (joinedThreadIds.isNotEmpty()) guildStore.setJoinedThreads(joinedThreadIds)
        } else {
            val channelIdSet = channelIds.toSet()
            val channelsById = entityStore.channels.value
            val scopedRemoved = current.filterNot { id -> channelsById[id]?.parent_id in channelIdSet }.toSet()
            guildStore.removeJoinedThreads(current - scopedRemoved)
            if (joinedThreadIds.isNotEmpty()) guildStore.setJoinedThreads(joinedThreadIds)
        }
    }

    private fun handleThreadMemberUpdate(data: JsonElement) {
        // Current user thread membership updated => joined/present in this thread.
        val obj = data.jsonObject
        val threadId = obj["id"]?.jsonPrimitive?.contentOrNull
            ?: obj["thread_id"]?.jsonPrimitive?.contentOrNull
        if (threadId != null) guildStore.setJoinedThreads(listOf(threadId))
    }

    private fun handleThreadMembersUpdate(data: JsonElement) {
        val obj = data.jsonObject
        val threadId = obj["id"]?.jsonPrimitive?.contentOrNull ?: return
        val me = userStore.currentUser.value?.id

        val removed = obj["removed_member_ids"]?.jsonArray?.mapNotNull { it.jsonPrimitive.contentOrNull } ?: emptyList()
        if (me != null && me in removed) {
            guildStore.removeJoinedThreads(listOf(threadId))
            return
        }

        val addedMembers = obj["added_members"]?.jsonArray ?: JsonArray(emptyList())
        val addedMe = if (me == null) false else addedMembers.any { el ->
            val m = el.jsonObject
            (m["user_id"]?.jsonPrimitive?.contentOrNull ?: m["member"]?.jsonObject?.get("user")?.jsonObject?.get("id")?.jsonPrimitive?.contentOrNull) == me
        }
        if (addedMe) guildStore.setJoinedThreads(listOf(threadId))
    }
}
