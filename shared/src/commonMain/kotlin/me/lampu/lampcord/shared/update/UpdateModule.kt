package me.lampu.lampcord.shared.update

import io.ktor.client.HttpClient
import kotlinx.serialization.json.Json
import org.koin.core.module.Module
import org.koin.dsl.module

object UpdateModels {
    val json: Json = Json { ignoreUnknownKeys = true; coerceInputValues = true }
}

fun updateModule(): Module = module {
    single {
        val target = currentUpdateTarget()
            ?: error("No update target for this platform")
        UpdateManager(
            client = get<HttpClient>(),
            json = UpdateModels.json,
            currentVersion = { APP_VERSION },
            target = target,
        )
    }
}
