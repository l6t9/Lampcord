package com.example.lampcord.shared.di

import com.example.lampcord.shared.api.DiscordClient
import com.example.lampcord.shared.api.createHttpClient
import com.example.lampcord.shared.gateway.GatewayManager
import com.example.lampcord.shared.state.*
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
        } 
    }
    single { createHttpClient() }
    single { DiscordClient(get(), get()) }
    single { GatewayManager(get(), get()) }
    
    single { ReadStateStore(get()) }
    single { PresenceStore(get()) }
    single { UserStore() }
    single { GuildStore() }
    single { MemberListStore() }
    single { MessageStore(get(), CoroutineScope(Dispatchers.Main)) }
    single { TokenStore(get()) }
    single { SettingsStore(get()) }

    single { 
        ChatState(
            get(), get(), get(), 
            get(), get(), get(), get(), get(), get(), get(), get()
        ) 
    }
}
