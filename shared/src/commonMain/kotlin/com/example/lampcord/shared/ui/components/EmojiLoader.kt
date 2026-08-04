package com.example.lampcord.shared.ui.components

import com.example.lampcord.shared.model.Emoji
import com.example.lampcord.shared.utils.EmojiIndex

object EmojiLoader {
    suspend fun getDefaultEmojis(): List<Emoji> {
        EmojiIndex.initialize()
        return EmojiIndex.getAllEmojis()
    }
}
