package com.example.materialcord.shared.di

import com.example.materialcord.shared.api.DiscordClient
import com.example.materialcord.shared.api.createHttpClient
import com.example.materialcord.shared.gateway.GatewayManager
import com.example.materialcord.shared.state.ChatState
import kotlinx.serialization.json.Json
import org.koin.dsl.module

val appModule = module {
    single { 
        Json { 
            ignoreUnknownKeys = true 
            coerceInputValues = true
        } 
    }
    single { createHttpClient() }
    single { DiscordClient(get()) }
    single { GatewayManager(get(), get()) }
    single { ChatState(get(), get(), get()) }
}
