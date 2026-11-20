package me.lampu.lampcord.shared.state

import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PendingQueueClearTest {

    private class FakeQueue : PendingSendQueue {
        var cancelAllCalls = 0
            private set

        override fun enqueue(task: MessageTask) = Unit
        override fun process() = Unit
        override fun cancel(nonce: String) = Unit
        override fun retry(task: MessageTask) = Unit
        override fun cancelAll() { cancelAllCalls++ }
    }

    @Test
    fun clearCancelsAndRemovesEveryPendingQueue() = runBlocking {
        val first = FakeQueue()
        val second = FakeQueue()
        val queues: MutableMap<String, PendingSendQueue> =
            mutableMapOf("channel-1" to first, "channel-2" to second)

        clearPendingQueues(queues, Mutex())

        assertTrue(queues.isEmpty())
        assertEquals(1, first.cancelAllCalls)
        assertEquals(1, second.cancelAllCalls)
    }
}
