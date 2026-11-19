package me.lampu.lampcord.shared.ui.components.profiles

import kotlin.test.Test
import kotlin.test.assertEquals

class ProfileBadgeIconUrlTest {
    @Test
    fun `legacy hash uses badge icons png`() {
        val hash = "a".repeat(32)
        assertEquals(
            "https://cdn.discordapp.com/badge-icons/$hash.png?size=64",
            profileBadgeIconUrl(hash),
        )
    }

    @Test
    fun `new leveled hash uses assets content svg`() {
        val hash = "b".repeat(64)
        assertEquals(
            "https://cdn.discordapp.com/assets/content/$hash.svg",
            profileBadgeIconUrl("$hash.svg"),
        )
    }

    @Test
    fun `explicit cdn paths and absolute urls pass through`() {
        val hash = "c".repeat(64)
        assertEquals(
            "https://cdn.discordapp.com/assets/content/$hash.svg",
            profileBadgeIconUrl("assets/content/$hash.svg"),
        )
        assertEquals(
            "https://example.com/badge.svg",
            profileBadgeIconUrl("https://example.com/badge.svg"),
        )
    }
}
