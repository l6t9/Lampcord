package me.lampu.lampcord.shared.model

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import me.lampu.lampcord.shared.utils.ResourceLoader

class EmojiTest {
    @Test
    fun twemojiVariationSelectorsMatchAssetNames() {
        assertEquals("$TWEMOJI_CDN_BASE_URL/2764.png", "❤️".toTwemojiUrl())
        assertEquals("$TWEMOJI_CDN_BASE_URL/1f636-200d-1f32b-fe0f.png", "😶‍🌫️".toTwemojiUrl())
        assertEquals("$TWEMOJI_CDN_BASE_URL/1f441-200d-1f5e8.png", "👁️‍🗨️".toTwemojiUrl())
    }

    @Test
    fun bundledTwemojiIsAvailable() {
        val bytes = assertNotNull(ResourceLoader.readBytes("twemoji/72x72/1f600.png"))
        assertEquals("PNG", bytes.copyOfRange(1, 4).decodeToString())
    }

    @Test
    fun customEmojiUrlUsesGifForAnimatedAndWebpForStatic() {
        // Matches Discord's own format, /emojis/{id}.{ext}?size={n}&quality=lossless.
        assertEquals(
            "https://cdn.discordapp.com/emojis/123.gif?size=64&quality=lossless",
            customEmojiCdnUrl("123", animated = true)
        )
        assertEquals(
            "https://cdn.discordapp.com/emojis/123.webp?size=64&quality=lossless",
            customEmojiCdnUrl("123", animated = false)
        )
        assertEquals(
            "https://cdn.discordapp.com/emojis/123.webp?size=32&quality=lossless",
            customEmojiCdnUrl("123", animated = false, size = 32)
        )
    }

    @Test
    fun displayUrlPrefersGifOnlyWhenAnimated() {
        assertEquals(
            "https://cdn.discordapp.com/emojis/123.gif?size=64&quality=lossless",
            Emoji(id = "123", name = "party").getDisplayUrl()
        )
        assertEquals(
            "https://cdn.discordapp.com/emojis/123.webp?size=64&quality=lossless",
            Emoji(id = "123", name = "smile").getDisplayUrl()
        )
    }
}
