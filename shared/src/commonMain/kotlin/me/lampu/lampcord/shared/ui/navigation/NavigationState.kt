package me.lampu.lampcord.shared.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSerializable
import androidx.compose.runtime.setValue
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.rememberDecoratedNavEntries
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.runtime.rememberSaveableStateHolderNavEntryDecorator
import androidx.savedstate.compose.serialization.serializers.MutableStateSerializer
import androidx.savedstate.serialization.SavedStateConfiguration
import kotlinx.serialization.KSerializer
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.polymorphic
import kotlinx.serialization.modules.subclass

internal val navigationSavedStateConfiguration =
    SavedStateConfiguration {
        val screenSerializer = Screen.serializer()
        val screenAsNavKeySerializer =
            object : KSerializer<NavKey> {
                override val descriptor = screenSerializer.descriptor

                override fun serialize(
                    encoder: Encoder,
                    value: NavKey,
                ) = screenSerializer.serialize(encoder, value as Screen)

                override fun deserialize(decoder: Decoder): NavKey = screenSerializer.deserialize(decoder)
            }
        serializersModule =
            SerializersModule {
                polymorphic(NavKey::class) {
                    subclass(Screen::class, screenSerializer)
                    subclass(Screen.Chat::class, Screen.Chat.serializer())
                    subclass(Screen.Friends::class, Screen.Friends.serializer())
                    subclass(Screen.Settings::class, Screen.Settings.serializer())
                    subclass(Screen.ServerSettings::class, Screen.ServerSettings.serializer())
                    subclass(Screen.ChannelSettings::class, Screen.ChannelSettings.serializer())
                    subclass(Screen.QuickSwitcher::class, Screen.QuickSwitcher.serializer())
                    subclass(Screen.Search::class, Screen.Search.serializer())
                    subclass(Screen.GlobalSearch::class, Screen.GlobalSearch.serializer())
                    subclass(Screen.Pins::class, Screen.Pins.serializer())
                    subclass(Screen.ChannelsAndRoles::class, Screen.ChannelsAndRoles.serializer())
                    subclass(Screen.Mentions::class, Screen.Mentions.serializer())
                    subclass(Screen.EasterEgg::class, Screen.EasterEgg.serializer())
                    subclass(Screen.MediaPicker::class, Screen.MediaPicker.serializer())
                    subclass(Screen.EmojiPicker::class, Screen.EmojiPicker.serializer())
                    subclass(Screen.Threads::class, Screen.Threads.serializer())
                    subclass(Screen.AccountSettings::class, Screen.AccountSettings.serializer())
                    subclass(Screen.ProfilesSettings::class, Screen.ProfilesSettings.serializer())
                    subclass(Screen.AppearanceSettings::class, Screen.AppearanceSettings.serializer())
                    subclass(Screen.AccessibilitySettings::class, Screen.AccessibilitySettings.serializer())
                    subclass(Screen.PrivacySettings::class, Screen.PrivacySettings.serializer())
                    subclass(Screen.ConnectionsSettings::class, Screen.ConnectionsSettings.serializer())
                    subclass(Screen.DevicesSettings::class, Screen.DevicesSettings.serializer())
                    subclass(Screen.ChatSettings::class, Screen.ChatSettings.serializer())
                    subclass(Screen.NotificationsSettings::class, Screen.NotificationsSettings.serializer())
                    subclass(Screen.AdvancedSettings::class, Screen.AdvancedSettings.serializer())
                    subclass(Screen.AdditionalSettings::class, Screen.AdditionalSettings.serializer())
                    subclass(Screen.UpdatesSettings::class, Screen.UpdatesSettings.serializer())
                    subclass(Screen.AboutSettings::class, Screen.AboutSettings.serializer())
                    subclass(Screen.Theming::class, Screen.Theming.serializer())
                    subclass(Screen.NavigationSettings::class, Screen.NavigationSettings.serializer())
                    subclass(Screen.ThemeEditor::class, Screen.ThemeEditor.serializer())
                }
                polymorphicDefaultSerializer(NavKey::class) { route ->
                    if (route is Screen) screenAsNavKeySerializer else null
                }
                polymorphicDefaultDeserializer(NavKey::class) { _ ->
                    screenAsNavKeySerializer
                }
            }
    }

@Composable
inline fun <reified K : NavKey> androidx.navigation3.runtime.EntryProviderScope<NavKey>.entry(
    noinline contentKey: (K) -> Any = { it.toString() },
    metadata: Map<String, Any> = emptyMap(),
    noinline content: @Composable (K) -> Unit
) {
    @Suppress("UNCHECKED_CAST")
    addEntryProvider(K::class, contentKey as (NavKey) -> Any, metadata, content as @Composable (NavKey) -> Unit)
}

@Composable
fun rememberNavigationState(
    startRoute: Screen,
    topLevelRoutes: Set<Screen>,
): NavigationState {
    val topLevelRoute =
        rememberSerializable(
            startRoute,
            topLevelRoutes,
            serializer = MutableStateSerializer(Screen.serializer()),
        ) {
            mutableStateOf(startRoute)
        }

    val backStacks =
        topLevelRoutes.associateWith { key ->
            rememberNavBackStack(navigationSavedStateConfiguration, key)
        }

    return remember(startRoute, topLevelRoutes) {
        NavigationState(
            startRoute = startRoute,
            topLevelRoute = topLevelRoute,
            backStacks = backStacks,
        )
    }
}

class NavigationState(
    startRoute: Screen,
    topLevelRoute: MutableState<Screen>,
    val backStacks: Map<Screen, NavBackStack<NavKey>>,
) {
    var startRoute: Screen by mutableStateOf(startRoute)
    var topLevelRoute: Screen by topLevelRoute
    val stacksInUse: List<Screen>
        get() =
            if (topLevelRoute == startRoute) {
                listOf(startRoute)
            } else {
                listOf(startRoute, topLevelRoute)
            }
}

@Composable
fun NavigationState.toEntries(entryProvider: (NavKey) -> NavEntry<NavKey>): List<NavEntry<NavKey>> {
    val decoratedEntries =
        backStacks.mapValues { (_, stack) ->
            val decorators =
                listOf(
                    rememberSaveableStateHolderNavEntryDecorator<NavKey>(),
                )
            rememberDecoratedNavEntries(
                backStack = stack,
                entryDecorators = decorators,
                entryProvider = entryProvider,
            )
        }

    return stacksInUse
        .flatMap { decoratedEntries[it] ?: emptyList() }
}
