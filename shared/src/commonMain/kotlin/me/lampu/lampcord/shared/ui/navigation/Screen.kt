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
    data object GlobalSearch : Screen()

    @Serializable
    data object Pins : Screen()
    
    @Serializable
    data object ChannelsAndRoles : Screen()
    
    @Serializable
    data object Mentions : Screen()
    
    @Serializable
    data object EasterEgg : Screen()
    
    @Serializable
    data object MediaPicker : Screen()
    
    @Serializable
    data object EmojiPicker : Screen()

    // Settings sub-screens
    @Serializable
    data object AccountSettings : Screen()

    @Serializable
    data object ProfilesSettings : Screen()

    @Serializable
    data object AppearanceSettings : Screen()

    @Serializable
    data object AccessibilitySettings : Screen()

    @Serializable
    data object PrivacySettings : Screen()

    @Serializable
    data object ConnectionsSettings : Screen()

    @Serializable
    data object DevicesSettings : Screen()

    @Serializable
    data object ChatSettings : Screen()

    @Serializable
    data object NotificationsSettings : Screen()

    @Serializable
    data object AdvancedSettings : Screen()

    @Serializable
    data object AboutSettings : Screen()

    @Serializable
    data object Theming : Screen()

    @Serializable
    data object NavigationSettings : Screen()

    @Serializable
    data class ThemeEditor(val themeJson: String) : Screen()
}
