package me.lampu.lampcord.shared.state

import me.lampu.lampcord.shared.settings.ThemeMode
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DiscordAppearanceSyncTest {

    @Test
    fun mapsDiscordThemesToLocalAppearance() {
        assertEquals(
            DiscordAppearance(ThemeMode.DARK, false),
            SettingsStore.mapDiscordAppearance("dark", syncAppearance = true)
        )
        assertEquals(
            DiscordAppearance(ThemeMode.LIGHT, false),
            SettingsStore.mapDiscordAppearance("light", syncAppearance = true)
        )
        assertEquals(
            DiscordAppearance(ThemeMode.DARK, true),
            SettingsStore.mapDiscordAppearance("darker", syncAppearance = true)
        )
        assertEquals(
            DiscordAppearance(ThemeMode.DARK, true),
            SettingsStore.mapDiscordAppearance("midnight", syncAppearance = true)
        )
        assertEquals(
            DiscordAppearance(ThemeMode.DARK, true),
            SettingsStore.mapDiscordAppearance("MIDNIGHT", syncAppearance = true)
        )
    }

    @Test
    fun ignoresNullUnknownThemesAndDisabledSync() {
        assertNull(SettingsStore.mapDiscordAppearance(null, syncAppearance = true))
        assertNull(SettingsStore.mapDiscordAppearance("dusk", syncAppearance = true))
        assertNull(SettingsStore.mapDiscordAppearance("dark", syncAppearance = false))
        assertNull(SettingsStore.mapDiscordAppearance("midnight", syncAppearance = false))
    }

    @Test
    fun pureBlackIsSentAsDiscordMidnight() {
        assertEquals("midnight", SettingsStore.DISCORD_THEME_PURE_BLACK)
        assertEquals(
            SettingsStore.DISCORD_THEME_PURE_BLACK,
            SettingsStore.discordThemeRawValue(ThemeMode.DARK, pureBlack = true)
        )
        assertEquals("dark", SettingsStore.discordThemeRawValue(ThemeMode.DARK, pureBlack = false))
        assertEquals("dark", SettingsStore.discordThemeRawValue(ThemeMode.AUTO, pureBlack = false))
        assertEquals("light", SettingsStore.discordThemeRawValue(ThemeMode.LIGHT, pureBlack = false))
    }
}
