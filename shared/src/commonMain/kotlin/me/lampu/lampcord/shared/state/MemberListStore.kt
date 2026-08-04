package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import me.lampu.lampcord.shared.model.*

class MemberListStore {
    val memberListItems = mutableStateMapOf<Int, MemberListListItem?>()
    val memberListGroups = mutableStateListOf<MemberListGroup>()
    
    val memberListRowCount by derivedStateOf {
        val groups = memberListGroups
        if (groups.isEmpty()) 0
        else groups.sumOf { it.count ?: it.member_count ?: 0 } + groups.size
    }

    fun clear() {
        memberListItems.clear()
        memberListGroups.clear()
    }

    fun handleMemberListUpdate(update: MemberListUpdate) {
        update.groups?.let { newGroups ->
            memberListGroups.clear()
            memberListGroups.addAll(newGroups)
        }

        for (op in update.ops) {
            when (op.op) {
                "SYNC" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val items = op.items ?: emptyList()
                    for (i in items.indices) {
                        memberListItems[start + i] = items[i]
                    }
                }
                "INSERT" -> {
                    val index = op.index ?: continue
                    val item = op.item ?: continue
                    val keysToShift = memberListItems.keys.filter { it >= index }.sortedDescending()
                    for (key in keysToShift) {
                        val value = memberListItems.remove(key)
                        if (value != null) {
                            memberListItems[key + 1] = value
                        }
                    }
                    memberListItems[index] = item
                }
                "UPDATE" -> {
                    val index = op.index ?: continue
                    val item = op.item ?: continue
                    memberListItems[index] = item
                }
                "DELETE" -> {
                    val index = op.index ?: continue
                    memberListItems.remove(index)
                    val keysToShift = memberListItems.keys.filter { it > index }.sorted()
                    for (key in keysToShift) {
                        val value = memberListItems.remove(key)
                        if (value != null) {
                            memberListItems[key - 1] = value
                        }
                    }
                }
                "INVALIDATE" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val end = range[1]
                    for (i in start..end) {
                        memberListItems.remove(i)
                    }
                }
            }
        }
    }
}
