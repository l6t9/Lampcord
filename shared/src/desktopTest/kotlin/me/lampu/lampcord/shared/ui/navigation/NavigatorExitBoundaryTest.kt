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
                    Screen.You to NavBackStack<NavKey>(Screen.You),
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

    @Test
    fun `root tabs have no in-stack back even when canGoBack is true`() {
        val state = newState()
        val navigator = Navigator(state)

        navigator.navigate(Screen.Friends)
        assertTrue(navigator.canGoBack)
        assertFalse(navigator.canGoBackInStack)

        navigator.navigate(Screen.AccountSettings)
        assertTrue(navigator.canGoBackInStack)
        assertTrue(navigator.goBack())
        assertEquals(Screen.Friends, state.topLevelRoute)
        assertFalse(navigator.canGoBackInStack)
    }

    @Test
    fun `back from You returns to the start route in one step`() {
        val state = newState()
        val navigator = Navigator(state)

        navigator.navigate(Screen.You)
        assertEquals(Screen.You, state.topLevelRoute)
        assertTrue(navigator.canGoBack)
        assertTrue(navigator.goBack())
        assertEquals(Screen.Chat, state.topLevelRoute)
    }

    @Test
    fun `back from settings returns to the route that opened it`() {
        val state = newState()
        val navigator = Navigator(state)

        navigator.navigate(Screen.You)
        navigator.navigate(Screen.Settings)

        assertTrue(navigator.goBack())
        assertEquals(Screen.You, state.topLevelRoute)
    }

    @Test
    fun `returning from settings leaves You stack clean for one-step back`() {
        val state = newState()
        val navigator = Navigator(state)

        navigator.navigate(Screen.You)
        navigator.navigate(Screen.Settings)
        navigator.navigate(Screen.You)

        assertEquals(1, state.backStacks.getValue(Screen.You).size)
        assertEquals(Screen.You, state.backStacks.getValue(Screen.You).last())
        assertTrue(navigator.goBack())
        assertEquals(Screen.Chat, state.topLevelRoute)
    }

    @Test
    fun `goBackTo pops the root stack to the explicit route`() {
        val state = newState()
        val navigator = Navigator(state)

        navigator.navigate(Screen.You)
        navigator.navigate(Screen.Settings)

        assertTrue(navigator.goBackTo(Screen.You))
        assertEquals(Screen.You, state.topLevelRoute)
        assertFalse(navigator.canGoBackInStack)
    }
}
