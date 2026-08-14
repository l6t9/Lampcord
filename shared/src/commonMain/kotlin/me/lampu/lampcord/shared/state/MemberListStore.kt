package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*

class MemberListStore(
    private val gatewayManager: GatewayManager,
    private val selectionStore: SelectionStore,
    private val userStore: UserStore,
    private val presenceStore: PresenceStore
) {
    // Member lists are cached per (guildId, listId) and kept alive across guild
    // switches, matching discord-jadx's StoreChannelMembers. Only logout clears them.
    private val guildCaches = mutableMapOf<String, MutableMap<String, MemberListCacheEntry>>()

    val memberListItems = mutableStateListOf<MemberListListItem?>()
    val memberListGroups = mutableStateMapOf<String, MemberListGroup>()
    var onlineCount by mutableStateOf<Int?>(null)
    var memberCount by mutableStateOf<Int?>(null)
    
    private var currentGuildId: String? = null
    private var currentListId: String? = null
    private var lastRanges: List<List<Int>> = emptyList()
    
    val memberListRowCount get() = memberListItems.size

    private class MemberListCacheEntry(
        val items: MutableList<MemberListListItem?>,
        val groups: MutableMap<String, MemberListGroup>,
        var onlineCount: Int? = null,
        var memberCount: Int? = null
    )

    private fun cacheFor(guildId: String): MutableMap<String, MemberListCacheEntry> =
        guildCaches.getOrPut(guildId) { mutableMapOf() }

    fun clear() {
        memberListItems.clear()
        memberListGroups.clear()
        guildCaches.clear()
        currentGuildId = null
        currentListId = null
        onlineCount = null
        memberCount = null
        lastRanges = emptyList()
    }

    fun requestMemberListRange(ranges: List<List<Int>>) {
        val guild = selectionStore.selectedGuild ?: return
        val channel = selectionStore.selectedChannel ?: return
        if (ranges.isNotEmpty()) lastRanges = ranges
        gatewayManager.sendLazyRequest(guild.id, channel.id, ranges)
    }

    // Re-send the subscription for the current channel after a reconnect. A fresh
    // gateway session starts with no server-side subscriptions, so the previously
    // cached list would otherwise go stale.
    fun resubscribe() {
        val guild = selectionStore.selectedGuild ?: return
        val channel = selectionStore.selectedChannel ?: return
        gatewayManager.sendLazyRequest(guild.id, channel.id, lastRanges.ifEmpty { listOf(listOf(0, 99)) })
    }

    fun setExpectedId(guildId: String, id: String, initialSize: Int) {
        // Already displaying this guild's list.
        if (currentGuildId == guildId && currentListId == id) return
        
        // Persist the currently displayed list into its per-guild cache entry.
        if (currentGuildId != null && currentListId != null) {
            val old = cacheFor(currentGuildId!!).getOrPut(currentListId!!) {
                MemberListCacheEntry(mutableListOf(), mutableMapOf())
            }
            if (memberListItems.isNotEmpty()) {
                old.items.clear()
                old.items.addAll(memberListItems)
                old.groups.clear()
                old.groups.putAll(memberListGroups)
                old.onlineCount = onlineCount
                old.memberCount = memberCount
            }
        }

        currentGuildId = guildId
        currentListId = id

        memberListItems.clear()
        memberListGroups.clear()
        onlineCount = null
        memberCount = null

        val cached = cacheFor(guildId)[id]
        if (cached != null) {
            memberListItems.addAll(cached.items)
            memberListGroups.putAll(cached.groups)
            onlineCount = cached.onlineCount
            memberCount = cached.memberCount
        } else if (initialSize > 0) {
            memberListItems.addAll(List(initialSize) { null })
        }
    }

    fun handleMemberListUpdate(update: MemberListUpdate) {
        // An update only touches the live list when it belongs to the guild/list
        // currently on screen. Everything else goes to that list's cache entry so
        // background guilds keep their own data (list ids like "everyone" repeat
        // across every guild).
        val isCurrent = update.guild_id == currentGuildId && update.id == currentListId

        val entry = cacheFor(update.guild_id).getOrPut(update.id) {
            MemberListCacheEntry(mutableListOf(), mutableMapOf(), update.online_count, update.member_count)
        }

        if (isCurrent) {
            update.online_count?.let { onlineCount = it }
            update.member_count?.let { memberCount = it }
        }
        update.online_count?.let { entry.onlineCount = it }
        update.member_count?.let { entry.memberCount = it }

        val targetItems: MutableList<MemberListListItem?> = if (isCurrent) memberListItems else entry.items
        val targetGroups: MutableMap<String, MemberListGroup> = if (isCurrent) memberListGroups else entry.groups

        for (op in update.ops) {
            when (op.op) {
                "SYNC" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val items = op.items ?: emptyList()
                    
                    val end = start + items.size
                    while (targetItems.size < end) targetItems.add(null)

                    items.forEachIndexed { i, item ->
                        val index = start + i
                        
                        item.member?.let { m ->
                            val userId = m.userId()
                            if (userId != null) {
                                userStore.cacheMember(update.guild_id, userId, m)
                                val pWithId = if (m.presence != null) {
                                    val p = m.presence
                                    if (p.user?.id == null && p.user_id == null) {
                                        p.copy(user_id = userId, guild_id = update.guild_id)
                                    } else p.copy(guild_id = update.guild_id)
                                } else {
                                    PresenceUpdate(user_id = userId, guild_id = update.guild_id, status = "offline")
                                }
                                presenceStore.handlePresenceUpdate(pWithId)
                            }
                        }

                        if (index < targetItems.size) {
                            targetItems[index] = item
                        }
                    }
                }
                "INSERT" -> {
                    val index = op.index ?: continue
                    val item = op.item ?: continue
                    item.member?.let { m ->
                        val userId = m.userId()
                        if (userId != null) {
                            userStore.cacheMember(update.guild_id, userId, m)
                            val pWithId = if (m.presence != null) {
                                val p = m.presence
                                if (p.user?.id == null && p.user_id == null) {
                                    p.copy(user_id = userId, guild_id = update.guild_id)
                                } else p.copy(guild_id = update.guild_id)
                            } else {
                                PresenceUpdate(user_id = userId, guild_id = update.guild_id, status = "offline")
                            }
                            presenceStore.handlePresenceUpdate(pWithId)
                        }
                    }
                    if (index <= targetItems.size) targetItems.add(index, item)
                }
                "UPDATE" -> {
                    val index = op.index ?: continue
                    val item = op.item ?: continue
                    item.member?.let { m ->
                        val userId = m.userId()
                        if (userId != null) {
                            userStore.cacheMember(update.guild_id, userId, m)
                            val pWithId = if (m.presence != null) {
                                val p = m.presence
                                if (p.user?.id == null && p.user_id == null) {
                                    p.copy(user_id = userId, guild_id = update.guild_id)
                                } else p.copy(guild_id = update.guild_id)
                            } else {
                                PresenceUpdate(user_id = userId, guild_id = update.guild_id, status = "offline")
                            }
                            presenceStore.handlePresenceUpdate(pWithId)
                        }
                    }
                    if (index < targetItems.size) targetItems[index] = item
                }
                "DELETE" -> {
                    val index = op.index ?: continue
                    if (index < targetItems.size) targetItems.removeAt(index)
                }
                "INVALIDATE" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val end = range[1]
                    for (i in start..end) if (i < targetItems.size) targetItems[i] = null
                }
            }
        }

        update.groups?.let { groups ->
            val totalSize = groups.sumOf { (it.count ?: it.member_count ?: 0) + 1 }
            
            // 126.21 Parity: Re-size the list to match the new group structure
            if (targetItems.size < totalSize) {
                repeat(totalSize - targetItems.size) { targetItems.add(null) }
            } else if (targetItems.size > totalSize) {
                while (targetItems.size > totalSize) targetItems.removeAt(targetItems.size - 1)
            }

            // Clear old group headers
            for (i in targetItems.indices) if (targetItems[i]?.group != null) targetItems[i] = null

            targetGroups.clear()
            var currentOffset = 0
            groups.forEach { group ->
                targetGroups[group.id] = group
                if (currentOffset < targetItems.size) {
                    targetItems[currentOffset] = MemberListListItem(group = group)
                }
                currentOffset += (group.count ?: group.member_count ?: 0) + 1
            }
        }
    }
}
