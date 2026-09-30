package me.lampu.lampcord.shared.api

import io.ktor.client.engine.HttpClientEngine

expect fun httpClientEngine(): HttpClientEngine
