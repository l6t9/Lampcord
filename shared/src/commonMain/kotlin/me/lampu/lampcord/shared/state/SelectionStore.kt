package me.lampu.lampcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild

class SelectionStore {
    private val scope = CoroutineScope(Dispatchers.Main)

    var selectedGuild by mutableStateOf<Guild?>(null)
    var selectedChannel by mutableStateOf<Channel?>(null)
    var selectedThread by mutableStateOf<Channel?>(null)

    val selectedGuildFlow: StateFlow<Guild?> = snapshotFlow { selectedGuild }
        .stateIn(scope, SharingStarted.Eagerly, null)

    val selectedChannelFlow: StateFlow<Channel?> = snapshotFlow { selectedChannel }
        .stateIn(scope, SharingStarted.Eagerly, null)

    val selectedThreadFlow: StateFlow<Channel?> = snapshotFlow { selectedThread }
        .stateIn(scope, SharingStarted.Eagerly, null)

    val activeChannelIdFlow = combine(selectedChannelFlow, selectedThreadFlow) { chan, thread ->
        thread?.id ?: chan?.id
    }.stateIn(scope, SharingStarted.Eagerly, null)

    fun clear() {
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
    }
}
