package me.lampu.lampcord.shared.utils

import kotlin.math.min
import kotlin.random.Random

class Backoff(
    private val minBackoffMs: Long = 500,
    private val maxBackoffMs: Long = 30000,
    private val jitter: Boolean = true
) {
    private var current = minBackoffMs
    private var fails = 0

    fun fail(): Long {
        fails++
        val multiplier = if (jitter) Random.nextDouble(0.5, 1.5) else 1.0
        current = min(current * 2, maxBackoffMs)
        return (current * multiplier).toLong()
    }

    fun succeed() {
        fails = 0
        current = minBackoffMs
    }

    fun nextDelay(): Long {
        val multiplier = if (jitter) Random.nextDouble(0.5, 1.5) else 1.0
        return (current * multiplier).toLong()
    }

    fun reset() {
        succeed()
    }

    fun isAtMax(): Boolean {
        return current >= maxBackoffMs
    }

    fun getFails(): Int {
        return fails
    }
}