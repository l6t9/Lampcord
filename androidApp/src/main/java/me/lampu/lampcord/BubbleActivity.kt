package me.lampu.lampcord

// MainActivity is singleTask, so bubbles need their own embeddable activity.
class BubbleActivity : MainActivity() {
    override val isBubble: Boolean = true
}
