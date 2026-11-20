package me.lampu.lampcord.shared.state

import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.model.MessageReaction
import me.lampu.lampcord.shared.model.ReactionCountDetails

/**
 * Pure helpers for reaction count updates. Discord's `count` includes super
 * (burst) reactions, so it must always equal normal + burst details.
 */
internal object ReactionCountMath {

    fun initial(emoji: Emoji, burst: Boolean, isMe: Boolean): MessageReaction {
        val details = if (burst) ReactionCountDetails(burst = 1, normal = 0)
        else ReactionCountDetails(burst = 0, normal = 1)
        return MessageReaction(
            emoji = emoji,
            count = details.normal + details.burst,
            count_details = details,
            me = !burst && isMe,
            me_burst = burst && isMe,
            burst_count = details.burst
        )
    }

    fun afterAdd(current: MessageReaction, burst: Boolean, isMe: Boolean): MessageReaction {
        val details = if (burst) {
            current.count_details.copy(burst = current.count_details.burst + 1)
        } else {
            current.count_details.copy(normal = current.count_details.normal + 1)
        }
        return current.copy(
            count = details.normal + details.burst,
            count_details = details,
            me = if (!burst) current.me || isMe else current.me,
            me_burst = if (burst) current.me_burst || isMe else current.me_burst,
            burst_count = details.burst
        )
    }

    fun afterRemove(current: MessageReaction, burst: Boolean, isMe: Boolean): MessageReaction {
        val normal = if (!burst) (current.count_details.normal - 1).coerceAtLeast(0)
        else current.count_details.normal
        val superCount = if (burst) (current.count_details.burst - 1).coerceAtLeast(0)
        else current.count_details.burst
        return current.copy(
            count = normal + superCount,
            count_details = ReactionCountDetails(burst = superCount, normal = normal),
            me = if (!burst && isMe) false else current.me,
            me_burst = if (burst && isMe) false else current.me_burst,
            burst_count = superCount
        )
    }
}
