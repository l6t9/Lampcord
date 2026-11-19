package me.lampu.lampcord.shared.ui.navigation

import androidx.compose.runtime.mutableStateOf
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class NavigatorExitBoundaryTest {
    private fun newState(): NavigationState =
        NavigationState(
            startRoute = Screen.Chat,
            topLevelRoute = mutableStateOf<Screen>(Screen.Chat),
            backStacks =
                mapOf(
                    Screen.Chat to NavBackStack<NavKey>(Screen.Chat),
                    Screen.Friends to NavBackStack<NavKey>(Screen.Friends),
                    Screen.Settings to NavBackStack<NavKey>(Screen.Settings),
                ),
        )

    @Test
    fun `start route cannot be popped`() {
        val navigator = Navigator(newState())

        assertFalse(navigator.canGoBack)
        assertFalse(navigator.goBack())
    }

    @Test
    fun `detail routes and non-start roots can be popped`() {
        val state = newState()
        val navigator = Navigator(state)

        navigator.navigate(Screen.AccountSettings)
        assertTrue(navigator.canGoBack)
        assertTrue(navigator.goBack())
        assertFalse(navigator.canGoBack)

        navigator.navigate(Screen.Settings)
        assertTrue(navigator.canGoBack)
        assertTrue(navigator.goBack())
        assertEquals(Screen.Chat, state.topLevelRoute)
    }
}
