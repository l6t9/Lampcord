package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.snapshots.SnapshotStateMap
import kotlinx.coroutines.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis

class TypingStore(private val scope: CoroutineScope) {
    // channelId -> userId -> timestamp
    val typingUsers = mutableStateMapOf<String, SnapshotStateMap<String, Long>>()

    fun handleTypingStart(channelId: String, userId: String, currentUserId: String?) {
        if (userId == currentUserId) return
        
        val channelTyping = typingUsers.getOrPut(channelId) { mutableStateMapOf() }
        channelTyping[userId] = getCurrentTimeMillis()
        
        // Remove after 10 seconds
        scope.launch {
            delay(10000L)
            if (channelTyping[userId] != null) {
                channelTyping.remove(userId)
            }
        }
    }

    fun clear() {
        typingUsers.clear()
    }
}
