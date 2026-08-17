package me.lampu.lampcord.shared.ui.navigation

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
sealed class Screen : NavKey {
    @Serializable
    data object Chat : Screen()

    @Serializable
    data object Friends : Screen()

    @Serializable
    data object Settings : Screen()

    @Serializable
    data object ServerSettings : Screen()

    @Serializable
    data object ChannelSettings : Screen()
    
    @Serializable
    data object QuickSwitcher : Screen()
    
    @Serializable
    data object Search : Screen()
    
    @Serializable
    data object Pins : Screen()
    
    @Serializable
    data object ChannelsAndRoles : Screen()
    
    @Serializable
    data object MediaPicker : Screen()
    
    @Serializable
    data object EmojiPicker : Screen()
}
