package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Message

class MentionsStore(
    private val messageApi: MessageApi,
    private val scope: CoroutineScope
) {
    val mentions = mutableStateListOf<Message>()
    var isLoading by mutableStateOf(false)
    var hasMore by mutableStateOf(true)

    fun loadMentions(refresh: Boolean = false) {
        if (isLoading) return
        if (refresh) {
            mentions.clear()
            hasMore = true
        }
        if (!hasMore) return

        isLoading = true
        scope.launch {
            val before = mentions.lastOrNull()?.id
            val newMentions = messageApi.getMentions(before = before)
            if (newMentions.isEmpty()) {
                hasMore = false
            } else {
                mentions.addAll(newMentions)
            }
            isLoading = false
        }
    }
}
