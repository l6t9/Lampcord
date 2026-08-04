package me.lampu.lampcord.shared.ui.components

import me.lampu.lampcord.shared.model.Emoji
import me.lampu.lampcord.shared.utils.EmojiIndex

object EmojiLoader {
    suspend fun getDefaultEmojis(): List<Emoji> {
        EmojiIndex.initialize()
        return EmojiIndex.getAllEmojis()
    }
}
