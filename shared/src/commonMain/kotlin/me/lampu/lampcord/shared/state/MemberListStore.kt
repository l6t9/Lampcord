package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.runtime.snapshots.SnapshotStateList
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*

class MemberListStore(
    private val gatewayManager: GatewayManager,
    private val selectionStore: SelectionStore
) {
    // 126.21 Parity: Cache multiple member lists by their memberListId
    // Map<memberListId, Entry>
    private val listCache = mutableMapOf<String, MemberListCacheEntry>()

    // Exposed state for the currently active list
    val memberListItems = mutableStateListOf<MemberListListItem?>()
    val memberListGroups = mutableStateMapOf<String, MemberListGroup>()
    var onlineCount by mutableStateOf<Int?>(null)
    var memberCount by mutableStateOf<Int?>(null)
    
    private var currentListId: String? = null
    
    val memberListRowCount get() = memberListItems.size

    data class MemberListCacheEntry(
        val items: MutableList<MemberListListItem?>,
        val groups: MutableMap<String, MemberListGroup>
    )

    fun clear() {
        memberListItems.clear()
        memberListGroups.clear()
        listCache.clear()
        currentListId = null
    }

    fun requestMemberListRange(ranges: List<List<Int>>) {
        val guild = selectionStore.selectedGuild ?: return
        val channel = selectionStore.selectedChannel ?: return
        gatewayManager.sendLazyRequest(guild.id, channel.id, ranges)
    }

    /**
     * 126.21 Parity: Pre-sizes the list based on the calculated member list ID.
     * Restores cached members immediately if they exist.
     */
    fun setExpectedId(id: String, initialSize: Int) {
        if (currentListId == id) return
        
        // Save current list to cache before switching if it has data
        currentListId?.let { oldId ->
            if (memberListItems.isNotEmpty()) {
                listCache[oldId] = MemberListCacheEntry(
                    items = memberListItems.toMutableList(),
                    groups = memberListGroups.toMutableMap()
                )
            }
        }

        memberListItems.clear()
        memberListGroups.clear()

        val cached = listCache[id]
        if (cached != null) {
            memberListItems.addAll(cached.items)
            memberListGroups.putAll(cached.groups)
        } else if (initialSize > 0) {
            // Pre-size with approximate count (126.21 StoreChannelMembers.getChannelMemberList)
            memberListItems.addAll(List(initialSize) { null })
        }
        
        currentListId = id
    }

    fun handleMemberListUpdate(update: MemberListUpdate) {
        if (update.id == currentListId) {
            update.online_count?.let { onlineCount = it }
            update.member_count?.let { memberCount = it }
        }

        // If the update is for a list we haven't seen, create a cache entry
        val entry = if (update.id == currentListId) {
            // Update current visible state
            null 
        } else {
            // Update background cache (Discord 126.21 updates all lists in parallel)
            listCache.getOrPut(update.id) {
                MemberListCacheEntry(mutableListOf(), mutableMapOf())
            }
        }

        val targetItems: MutableList<MemberListListItem?> = entry?.items ?: memberListItems
        val targetGroups: MutableMap<String, MemberListGroup> = entry?.groups ?: memberListGroups

        // 1. Handle Operations (Matches StoreChannelMembers.handleGuildMemberListUpdate order)
        for (op in update.ops) {
            when (op.op) {
                "SYNC" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val items = op.items ?: emptyList()
                    for (i in items.indices) {
                        val index = start + i
                        if (index < targetItems.size) {
                            targetItems[index] = items[i]
                        } else if (index >= targetItems.size) {
                            // Expand if SYNC goes beyond current size
                            while (targetItems.size <= index) targetItems.add(null)
                            targetItems[index] = items[i]
                        }
                    }
                }
                "INSERT" -> {
                    val index = op.index ?: continue
                    val item = op.item ?: continue
                    if (index <= targetItems.size) {
                        targetItems.add(index, item)
                    }
                }
                "UPDATE" -> {
                    val index = op.index ?: continue
                    val item = op.item ?: continue
                    if (index < targetItems.size) {
                        targetItems[index] = item
                    }
                }
                "DELETE" -> {
                    val index = op.index ?: continue
                    if (index < targetItems.size) {
                        targetItems.removeAt(index)
                    }
                }
                "INVALIDATE" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val end = range[1]
                    for (i in start..end) {
                        if (i < targetItems.size) {
                            targetItems[i] = null
                        }
                    }
                }
            }
        }

        // 2. Handle Groups (Matches ChannelMemberList.setGroups)
        update.groups?.let { groups ->
            val totalSize = groups.sumOf { (it.count ?: it.member_count ?: 0) + 1 }
            
            // Resize list to match total size from groups
            if (targetItems.size < totalSize) {
                repeat(totalSize - targetItems.size) { targetItems.add(null) }
            } else if (targetItems.size > totalSize) {
                while (targetItems.size > totalSize) {
                    targetItems.removeAt(targetItems.size - 1)
                }
            }

            // Clear existing headers to avoid duplicates when shifts occur
            for (i in targetItems.indices) {
                if (targetItems[i]?.group != null) {
                    targetItems[i] = null
                }
            }

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
