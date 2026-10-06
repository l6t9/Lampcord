package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.Snapshot
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*

class MemberListStore(
    private val gatewayManager: GatewayManager,
    private val selectionStore: SelectionStore,
    private val userStore: UserStore,
    private val presenceStore: PresenceStore
) {
    // Member lists are cached per (guildId, listId) and kept alive across guild switches, matching discord-jadx's StoreChannelMembers. Only logout clears them.
    private val guildCaches = mutableMapOf<String, MutableMap<String, MemberListCacheEntry>>()
    private val guildCacheOrder = mutableListOf<String>()

    private companion object {
        const val MAX_CACHED_GUILDS = 10
    }

    val memberListItems = mutableStateListOf<MemberListListItem?>()
    val memberListGroups = mutableStateMapOf<String, MemberListGroup>()
    var onlineCount by mutableStateOf<Int?>(null)
    var memberCount by mutableStateOf<Int?>(null)

    private val guildOnlineCounts = mutableStateMapOf<String, Int>()
    private val guildMemberCounts = mutableStateMapOf<String, Int>()

    fun getOnlineCount(guildId: String): Int = guildOnlineCounts[guildId] ?: 0
    fun getMemberCount(guildId: String): Int = guildMemberCounts[guildId] ?: 0
    
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

    private fun cacheFor(guildId: String): MutableMap<String, MemberListCacheEntry> {
        guildCacheOrder.remove(guildId)
        guildCacheOrder.add(guildId)
        while (guildCacheOrder.size > MAX_CACHED_GUILDS) {
            val oldest = guildCacheOrder.removeAt(0)
            if (oldest != guildId) guildCaches.remove(oldest)
        }
        return guildCaches.getOrPut(guildId) { mutableMapOf() }
    }

    fun clear() {
        memberListItems.clear()
        memberListGroups.clear()
        guildCaches.clear()
        guildCacheOrder.clear()
        guildOnlineCounts.clear()
        guildMemberCounts.clear()
        currentGuildId = null
        currentListId = null
        onlineCount = null
        memberCount = null
        lastRanges = emptyList()
    }

    fun requestMemberListRange(ranges: List<List<Int>>) {
        val guild = selectionStore.selectedGuild ?: return
        val thread = selectionStore.selectedThread
        val channel = selectionStore.selectedChannel ?: return
        
        if (ranges.isNotEmpty()) lastRanges = ranges
        
        if (thread != null) {
            gatewayManager.sendLazyRequest(guild.id, thread.id, thread.id, ranges, isThread = true)
        } else {
            val listId = currentListId ?: channel.memberListId(guild)
            gatewayManager.sendLazyRequest(guild.id, channel.id, listId, ranges, isThread = false)
        }
    }

    // Re-send the subscription for the current channel after a reconnect. A fresh gateway session starts with no server-side subscriptions, so the previously cached list would otherwise go stale.
    fun resubscribe() {
        val guild = selectionStore.selectedGuild ?: return
        val thread = selectionStore.selectedThread
        val channel = selectionStore.selectedChannel ?: return
        val ranges = lastRanges.take(2).ifEmpty { listOf(listOf(0, 99)) }
        if (thread != null) {
            gatewayManager.sendLazyRequest(guild.id, thread.id, thread.id, ranges, isThread = true)
        } else {
            val listId = currentListId ?: channel.memberListId(guild)
            gatewayManager.sendLazyRequest(guild.id, channel.id, listId, ranges, isThread = false)
        }
    }

    fun setExpectedId(guildId: String, id: String, initialSize: Int) {
        if (currentGuildId == guildId && currentListId == id) return
        
        val previousGuildId = currentGuildId
        val previousListId = currentListId
        if (previousGuildId != null && previousListId != null) {
            val old = cacheFor(previousGuildId).getOrPut(previousListId) {
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
        val diagnosticItems = update.ops.sumOf { (it.items?.size ?: 0) + (if (it.item != null) 1 else 0) }

        val isCurrent = update.guild_id == currentGuildId && update.id == currentListId

        val entry = cacheFor(update.guild_id).getOrPut(update.id) {
            MemberListCacheEntry(mutableListOf(), mutableMapOf(), update.online_count, update.member_count)
        }

        // Members and presences are collected first and pushed to their stores in one batch each, so a 100 member SYNC costs one map copy instead of one per member.
        val members = ArrayList<Pair<String, Member>>(diagnosticItems)
        val presences = ArrayList<PresenceUpdate>(diagnosticItems)
        fun collect(item: MemberListListItem) {
            val m = item.member ?: return
            val userId = m.userId() ?: return
            members.add(userId to m)
            val p = m.presence
            presences.add(
                when {
                    p == null -> PresenceUpdate(user_id = userId, guild_id = update.guild_id, status = "offline")
                    p.user?.id == null && p.user_id == null -> p.copy(user_id = userId, guild_id = update.guild_id)
                    else -> p.copy(guild_id = update.guild_id)
                }
            )
        }

        // One snapshot apply for the whole update instead of one per row write.
        Snapshot.withMutableSnapshot {
            if (isCurrent) {
                update.online_count?.let { onlineCount = it }
                update.member_count?.let { memberCount = it }
            }
            update.online_count?.let {
                entry.onlineCount = it
                guildOnlineCounts[update.guild_id] = it
            }
            update.member_count?.let {
                entry.memberCount = it
                guildMemberCounts[update.guild_id] = it
            }

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
                            collect(item)
                            targetItems[start + i] = item
                        }
                    }
                    "INSERT" -> {
                        val index = op.index ?: continue
                        val item = op.item ?: continue
                        collect(item)
                        if (index <= targetItems.size) targetItems.add(index, item)
                    }
                    "UPDATE" -> {
                        val index = op.index ?: continue
                        val item = op.item ?: continue
                        collect(item)
                        if (index < targetItems.size) targetItems[index] = item
                    }
                    "DELETE" -> {
                        val index = op.index ?: continue
                        if (index < targetItems.size) targetItems.removeAt(index)
                    }
                    "INVALIDATE" -> {
                        val range = op.range ?: continue
                        val end = minOf(range[1], targetItems.size - 1)
                        for (i in range[0]..end) targetItems[i] = null
                    }
                }
            }

            update.groups?.let { groups ->
                val totalSize = groups.sumOf { (it.count ?: it.member_count ?: 0) + 1 }

                if (targetItems.size < totalSize) {
                    repeat(totalSize - targetItems.size) { targetItems.add(null) }
                } else if (targetItems.size > totalSize) {
                    while (targetItems.size > totalSize) targetItems.removeAt(targetItems.size - 1)
                }

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

        userStore.cacheMembers(update.guild_id, members)
        presenceStore.applyPresences(presences)
    }
}
