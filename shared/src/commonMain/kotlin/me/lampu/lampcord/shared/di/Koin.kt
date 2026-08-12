package me.lampu.lampcord.shared.di

import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.api.RemoteAuthClient
import me.lampu.lampcord.shared.api.createHttpClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.state.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.serialization.json.Json
import org.koin.dsl.module

val networkModule = module {
    single { 
        Json { 
            ignoreUnknownKeys = true 
            coerceInputValues = true
            isLenient = true
            explicitNulls = false
        } 
    }
    single { createHttpClient() }
    single { DiscordClient(get(), get()) }
    single { RemoteAuthClient(get(), get()) }
    single { GatewayManager(get(), get(), get(), get()) }
    single { VoiceGatewayManager(get(), get()) }
}

val storeModule = module {
    single { UserStore() }
    single { EntityStore(get()) }
    single { SelectionStore() }
    single { ReadStateStore(get()) }
    single { UserGuildSettingsStore() }
    single { AppErrorStore() }
    single { PresenceStore(get()) }
    single { RelationshipStore(get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { GuildStore(get(), get(), get(), get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { MemberListStore(get(), get(), get(), get()) }
    single { MessageStore(get(), get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { TypingStore(CoroutineScope(Dispatchers.Main)) }
    single { VoiceStore(get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { SearchStore(get(), get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { ProfileStore(get(), CoroutineScope(Dispatchers.Main)) }
    single { AutocompleteStore(get(), get(), get(), get()) }
    single { CommandStore(get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { ExperimentStore() }
    single { BadgeStore(get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { FinderStore(get(), CoroutineScope(Dispatchers.Main)) }
    single { TokenStore(get()) }
    single { SettingsStore(get()) }
    single { NavigationStore(get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), getOrNull<me.lampu.lampcord.shared.notifications.MessageNotifier>(), CoroutineScope(Dispatchers.Main)) }
    single { NotificationStore(CoroutineScope(Dispatchers.Main)) }
    single { ChannelNavigator(get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { 
        SessionManager(
            get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get()
        )
    }

    single { me.lampu.lampcord.shared.state.handlers.MessageEventHandler(get(), get(), get(), get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { me.lampu.lampcord.shared.state.handlers.GuildEventHandler(get(), get(), get(), get(), get(), get()) }
    single { me.lampu.lampcord.shared.state.handlers.PresenceEventHandler(get(), get(), get()) }
    single { me.lampu.lampcord.shared.state.handlers.RelationshipEventHandler(get(), get()) }
    single { me.lampu.lampcord.shared.state.handlers.UserEventHandler(get(), get(), get()) }
    single { me.lampu.lampcord.shared.state.handlers.TypingEventHandler(get(), get(), get()) { get<UserStore>().currentUser.value?.id } }
    single { me.lampu.lampcord.shared.state.handlers.MemberListEventHandler(get(), get()) }
    single { me.lampu.lampcord.shared.state.handlers.NotificationEventHandler(get(), get(), get(), get(), get(), get(), get(), get(), getOrNull<me.lampu.lampcord.shared.notifications.MessageNotifier>()) }
    
    single {
        GatewayEventDispatcher(
            listOf(
                get<me.lampu.lampcord.shared.state.handlers.MessageEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.GuildEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.PresenceEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.RelationshipEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.UserEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.TypingEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.MemberListEventHandler>(),
                get<me.lampu.lampcord.shared.state.handlers.NotificationEventHandler>()
            )
        )
    }
}

val gatewayModule = module {
    single {
        GatewayHandler(
            get(), get(), CoroutineScope(Dispatchers.Main),
            get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get()
        )
    }
}

val appModule = module {
    includes(networkModule, storeModule, gatewayModule)
}
