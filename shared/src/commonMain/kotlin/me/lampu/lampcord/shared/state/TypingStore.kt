package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.snapshots.SnapshotStateMap
import kotlinx.coroutines.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis

class TypingStore(private val scope: CoroutineScope) {
    // channelId -> userId -> timestamp
    val typingUsers = mutableStateMapOf<String, SnapshotStateMap<String, Long>>()
    private val typingJobs = mutableMapOf<Pair<String, String>, Job>()

    fun handleTypingStart(channelId: String, userId: String, currentUserId: String?) {
        if (userId == currentUserId) return
        
        val channelTyping = typingUsers.getOrPut(channelId) { mutableStateMapOf() }
        channelTyping[userId] = getCurrentTimeMillis()
        
        typingJobs[channelId to userId]?.cancel()
        typingJobs[channelId to userId] = scope.launch {
            delay(10000L)
            channelTyping.remove(userId)
            typingJobs.remove(channelId to userId)
        }
    }

    fun clear() {
        typingUsers.clear()
        typingJobs.values.forEach { it.cancel() }
        typingJobs.clear()
    }
}
