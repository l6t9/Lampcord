package me.lampu.lampcord.shared.state

import me.lampu.lampcord.shared.model.Emoji
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ReactionCountMathTest {
    private val emoji = Emoji(id = "1", name = "wave")

    @Test
    fun newBurstReactionHasTotalCountOne() {
        val reaction = ReactionCountMath.initial(emoji, burst = true, isMe = true)
        assertEquals(1, reaction.count)
        assertEquals(1, reaction.count_details.burst)
        assertEquals(0, reaction.count_details.normal)
        assertEquals(1, reaction.burst_count)
        assertTrue(reaction.me_burst)
        assertFalse(reaction.me)
    }

    @Test
    fun burstAddIncrementsTotalCount() {
        val initial = ReactionCountMath.initial(emoji, burst = false, isMe = false)
        val updated = ReactionCountMath.afterAdd(initial, burst = true, isMe = true)
        assertEquals(2, updated.count)
        assertEquals(1, updated.count_details.normal)
        assertEquals(1, updated.count_details.burst)
        assertEquals(1, updated.burst_count)
        assertTrue(updated.me_burst)
        assertFalse(updated.me)
    }

    @Test
    fun burstRemoveKeepsNormalCountInTotal() {
        val initial = ReactionCountMath.initial(emoji, burst = false, isMe = false)
        val withBurst = ReactionCountMath.afterAdd(initial, burst = true, isMe = false)
        val removed = ReactionCountMath.afterRemove(withBurst, burst = true, isMe = false)
        assertEquals(1, removed.count)
        assertEquals(1, removed.count_details.normal)
        assertEquals(0, removed.count_details.burst)
        assertEquals(0, removed.burst_count)
    }

    @Test
    fun normalRemoveKeepsBurstCountInTotal() {
        val initial = ReactionCountMath.initial(emoji, burst = true, isMe = false)
        val withNormal = ReactionCountMath.afterAdd(initial, burst = false, isMe = false)
        val removed = ReactionCountMath.afterRemove(withNormal, burst = false, isMe = false)
        assertEquals(1, removed.count)
        assertEquals(0, removed.count_details.normal)
        assertEquals(1, removed.count_details.burst)
        assertEquals(1, removed.burst_count)
    }
}
