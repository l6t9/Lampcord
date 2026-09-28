package me.lampu.lampcord.shared.utils

import kotlin.random.Random

object Snowflake {
    private const val DISCORD_EPOCH = 1420070400000L

    fun nextId(): String {
        val timestamp = getCurrentTimeMillis()
        val sequence = Random.nextLong(0, 4096)
        
        val id = ((timestamp - DISCORD_EPOCH) shl 22) or
                 (1L shl 17) or // worker id 1
                 (1L shl 12) or // process id 1
                 sequence
        
        return id.toString()
    }
}
