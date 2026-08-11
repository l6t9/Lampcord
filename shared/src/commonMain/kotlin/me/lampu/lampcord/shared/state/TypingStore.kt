package me.lampu.lampcord.shared.state

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlin.time.Duration.Companion.milliseconds

class TypingStore(private val scope: CoroutineScope) {
    // channelId -> userId -> timestamp
    private val _typingUsers = MutableStateFlow<Map<String, Map<String, Long>>>(emptyMap())
    val typingUsers: StateFlow<Map<String, Map<String, Long>>> = _typingUsers.asStateFlow()
    
    private val typingJobs = mutableMapOf<Pair<String, String>, Job>()

    fun handleTypingStart(channelId: String, userId: String, currentUserId: String?) {
        if (userId == currentUserId) return
        
        _typingUsers.update { current ->
            val channelTyping = current[channelId]?.toMutableMap() ?: mutableMapOf()
            channelTyping[userId] = getCurrentTimeMillis()
            current + (channelId to channelTyping)
        }
        
        typingJobs[channelId to userId]?.cancel()
        typingJobs[channelId to userId] = scope.launch {
            delay(10000L.milliseconds)
            _typingUsers.update { current ->
                val channelTyping = current[channelId]?.toMutableMap() ?: return@update current
                channelTyping.remove(userId)
                if (channelTyping.isEmpty()) current - channelId else current + (channelId to channelTyping)
            }
            typingJobs.remove(channelId to userId)
        }
    }

    fun clear() {
        _typingUsers.value = emptyMap()
        typingJobs.values.forEach { it.cancel() }
        typingJobs.clear()
    }
}
