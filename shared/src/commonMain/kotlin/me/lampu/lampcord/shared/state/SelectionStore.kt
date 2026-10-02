package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateOf
import kotlinx.coroutines.flow.MutableStateFlow
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild

class SelectionStore {
    private val guildState = mutableStateOf<Guild?>(null)
    private val channelState = mutableStateOf<Channel?>(null)
    private val threadState = mutableStateOf<Channel?>(null)

    val selectedGuildFlow = MutableStateFlow<Guild?>(null)
    val selectedChannelFlow = MutableStateFlow<Channel?>(null)
    val selectedThreadFlow = MutableStateFlow<Channel?>(null)
    val activeChannelIdFlow = MutableStateFlow<String?>(null)

    var selectedGuild: Guild?
        get() = guildState.value
        set(value) {
            guildState.value = value
            selectedGuildFlow.value = value
        }

    var selectedChannel: Channel?
        get() = channelState.value
        set(value) {
            channelState.value = value
            selectedChannelFlow.value = value
            syncActiveChannelId()
        }

    var selectedThread: Channel?
        get() = threadState.value
        set(value) {
            threadState.value = value
            selectedThreadFlow.value = value
            syncActiveChannelId()
        }

    private fun syncActiveChannelId() {
        activeChannelIdFlow.value = threadState.value?.id ?: channelState.value?.id
    }

    fun clear() {
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
    }
}