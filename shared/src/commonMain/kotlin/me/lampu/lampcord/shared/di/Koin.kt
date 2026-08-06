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

val appModule = module {
    single { 
        Json { 
            ignoreUnknownKeys = true 
            coerceInputValues = true
            isLenient = true
            explicitNulls = true
        } 
    }
    single { createHttpClient() }
    single { DiscordClient(get(), get()) }
    single { GatewayManager(get(), get()) }
    single { VoiceGatewayManager(get(), get()) }
    single { RemoteAuthClient(get(), get()) }
    
    single { ReadStateStore(get()) }
    single { UserGuildSettingsStore() }
    single { PresenceStore(get()) }
    single { UserStore() }
    single { RelationshipStore(get(), CoroutineScope(Dispatchers.Main)) }
    single { GuildStore() }
    single { MemberListStore() }
    single { MessageStore(get(), CoroutineScope(Dispatchers.Main)) }
    single { TypingStore(CoroutineScope(Dispatchers.Main)) }
    single { CommandStore() }
    single { TokenStore(get()) }
    single { SettingsStore(get()) }

    single { 
        ChatState(
            get(), get(), get(), get(),
            get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get(), get()
        ) 
    }
}
