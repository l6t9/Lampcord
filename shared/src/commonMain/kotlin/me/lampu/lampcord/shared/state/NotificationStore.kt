package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.notifications.IncomingNotificationData

class NotificationStore(
    private val scope: CoroutineScope
) {
    data class InAppNotification(
        val id: String,
        val data: IncomingNotificationData
    )

    private val _toasts = MutableStateFlow<List<InAppNotification>>(emptyList())
    val toasts: StateFlow<List<InAppNotification>> = _toasts.asStateFlow()

    private val dismissJobs = mutableMapOf<String, Job>()

    fun show(data: IncomingNotificationData) {
        val id = data.message.id
        _toasts.update { current ->
            current.filterNot { it.id == id } + InAppNotification(id, data)
        }
        dismissJobs[id]?.cancel()
        dismissJobs[id] = scope.launch {
            delay(5000)
            dismiss(id)
        }
    }

    fun dismiss(id: String) {
        dismissJobs.remove(id)?.cancel()
        _toasts.update { current -> current.filterNot { it.id == id } }
    }

    fun dismissChannel(channelId: String) {
        _toasts.update { current ->
            current.filter { it.data.message.channel_id != channelId }
        }
    }

    fun dismissAll() {
        dismissJobs.keys.toList().forEach(::dismiss)
    }
}
