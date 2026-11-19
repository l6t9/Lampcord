package me.lampu.lampcord.shared.ui.navigation

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.navigation3.runtime.NavKey

class Navigator(
    val state: NavigationState,
) {
    fun navigate(route: NavKey) {
        val topLevelKey = state.backStacks.keys.find { it::class == route::class }
        if (topLevelKey != null) {
            if (state.topLevelRoute::class == route::class) {
                val stack = state.backStacks.getValue(topLevelKey)
                while (stack.size > 1) stack.removeLastOrNull()
            }
            state.topLevelRoute = topLevelKey
        } else {
            val stack = state.backStacks[state.topLevelRoute] ?: return
            if (stack.lastOrNull() != route) {
                stack.add(route)
            }
        }
    }

    fun goBack(): Boolean {
        val currentStack =
            state.backStacks[state.topLevelRoute]
                ?: error("Stack for ${state.topLevelRoute} not found")
        val currentRoute = currentStack.last()

        if (currentRoute == state.topLevelRoute) {
            if (state.topLevelRoute == state.startRoute) return false
            state.topLevelRoute = state.startRoute
        } else {
            currentStack.removeLastOrNull()
        }
        return true
    }
}
