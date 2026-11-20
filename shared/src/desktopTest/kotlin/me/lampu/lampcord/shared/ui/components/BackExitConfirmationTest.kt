package me.lampu.lampcord.shared.ui.components

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class BackExitConfirmationTest {
    @Test
    fun `two backs inside window exit`() {
        val confirmation = BackExitConfirmation()

        assertFalse(confirmation.onBack(1_000L))
        assertTrue(confirmation.onBack(2_999L))
    }

    @Test
    fun `backs outside window do not exit`() {
        val confirmation = BackExitConfirmation()

        assertFalse(confirmation.onBack(1_000L))
        assertFalse(confirmation.onBack(3_000L))
        assertTrue(confirmation.onBack(4_500L))
    }

    @Test
    fun `reset invalidates pending confirmation`() {
        val confirmation = BackExitConfirmation()

        assertFalse(confirmation.onBack(1_000L))
        confirmation.reset()
        assertFalse(confirmation.onBack(1_500L))
        assertTrue(confirmation.onBack(2_000L))
    }
}
