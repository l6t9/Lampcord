package me.lampu.lampcord.shared.ui.navigation

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.navigation3.runtime.NavKey

class Navigator(
    val state: NavigationState,
) {
    private val rootStack = mutableListOf(state.startRoute)

    val canGoBack: Boolean
        get() {
            val stack = state.backStacks[state.topLevelRoute] ?: return false
            return stack.isNotEmpty() &&
                (stack.lastOrNull() != state.topLevelRoute || state.topLevelRoute != state.startRoute)
        }

    val canGoBackInStack: Boolean
        get() {
            val stack = state.backStacks[state.topLevelRoute] ?: return false
            return stack.lastOrNull() != state.topLevelRoute
        }

    fun navigate(route: NavKey) {
        val topLevelKey = state.backStacks.keys.find { it::class == route::class }
        if (topLevelKey != null) {
            if (state.topLevelRoute::class == route::class) {
                val stack = state.backStacks.getValue(topLevelKey)
                while (stack.size > 1) stack.removeLastOrNull()
            }
            if (state.topLevelRoute != topLevelKey) {
                val existingIndex = rootStack.indexOf(topLevelKey)
                if (existingIndex >= 0) {
                    while (rootStack.size > existingIndex + 1) {
                        rootStack.removeLastOrNull()
                    }
                } else {
                    rootStack.add(topLevelKey)
                }
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
            val currentIndex = rootStack.lastIndexOf(state.topLevelRoute)
            if (currentIndex > 0) {
                while (rootStack.size > currentIndex) {
                    rootStack.removeLastOrNull()
                }
                state.topLevelRoute = rootStack.last()
            } else {
                rootStack.clear()
                rootStack.add(state.startRoute)
                state.topLevelRoute = state.startRoute
            }
        } else {
            currentStack.removeLastOrNull()
        }
        return true
    }

    fun goBackTo(route: Screen): Boolean {
        val index = rootStack.indexOf(route)
        if (index >= 0) {
            while (rootStack.size > index + 1) {
                rootStack.removeLastOrNull()
            }
            state.topLevelRoute = route
            return true
        }
        if (route == state.startRoute) {
            rootStack.clear()
            rootStack.add(state.startRoute)
            state.topLevelRoute = state.startRoute
            return true
        }
        return false
    }
}
