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
    single { GatewayManager(get(), get(), get()) }
    single { VoiceGatewayManager(get(), get()) }
}

val storeModule = module {
    single { SelectionStore() }
    single { ReadStateStore(get()) }
    single { UserGuildSettingsStore() }
    single { AppErrorStore() }
    single { PresenceStore(get()) }
    single { UserStore() }
    single { RelationshipStore(get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { GuildStore(get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { MemberListStore() }
    single { MessageStore(get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { TypingStore(CoroutineScope(Dispatchers.Main)) }
    single { VoiceStore(get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { SearchStore(get(), get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
    single { ProfileStore(get(), CoroutineScope(Dispatchers.Main)) }
    single { AutocompleteStore(get(), get(), get(), get()) }
    single { CommandStore() }
    single { ExperimentStore() }
    single { FinderStore(get()) }
    single { TokenStore(get()) }
    single { SettingsStore(get()) }
    single { NavigationStore(get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), CoroutineScope(Dispatchers.Main)) }
}

val gatewayModule = module {
    single {
        GatewayHandler(
            get(), get(), CoroutineScope(Dispatchers.Main),
            get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get()
        )
    }
}

val chatModule = module {
    single { 
        ChatState(
            get(), get(), get(), get(),
            get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get()
        ) 
    }
}

val appModule = module {
    includes(networkModule, storeModule, gatewayModule, chatModule)
}
