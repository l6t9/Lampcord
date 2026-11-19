package me.lampu.lampcord.shared.state

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class SettingsUpdateGuardTest {

    @Test
    fun onlyTheNewestUpdateMayRollBack() {
        val guard = SettingsUpdateGuard()
        val first = guard.begin()
        val second = guard.begin()

        assertFalse(guard.isLatest(first))
        assertTrue(guard.isLatest(second))
    }

    @Test
    fun aSinglePendingUpdateIsStillLatest() {
        val guard = SettingsUpdateGuard()
        val revision = guard.begin()
        assertTrue(guard.isLatest(revision))
    }
}
