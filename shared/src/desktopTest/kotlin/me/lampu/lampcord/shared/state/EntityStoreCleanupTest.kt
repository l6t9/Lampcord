package me.lampu.lampcord.shared.state

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotSame
import kotlin.test.assertTrue

class EntityStoreCleanupTest {

    @Test
    fun removeGuildEmptiesAndDropsItsPerGuildFlow() {
        val store = EntityStore(UserStore())
        store.updateGuild(
            Guild(id = "guild-1", name = "Guild").copy(
                channels = listOf(Channel(id = "channel-1", type = 0, guild_id = "guild-1"))
            )
        )

        val before = store.channelsForGuild("guild-1")
        assertEquals(1, before.value.size)

        store.removeGuild("guild-1")

        assertTrue(before.value.isEmpty())
        assertTrue(store.channels.value.values.none { it.guild_id == "guild-1" })
        // key removed: a fresh flow is created instead of reusing the old one
        assertNotSame(before, store.channelsForGuild("guild-1"))
    }
}
