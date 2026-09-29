package me.lampu.lampcord.shared.update

import io.ktor.client.HttpClient
import me.lampu.lampcord.shared.utils.Logging
import kotlinx.serialization.json.Json
import org.koin.core.module.Module
import org.koin.dsl.module

object UpdateModels {
    val json: Json = Json { ignoreUnknownKeys = true; coerceInputValues = true }
}

fun updateModule(): Module = module {
    single(createdAtStart = true) {
        val target = currentUpdateTarget()
            ?: error("No update target for this platform")
        Logging.i(
            "Updates",
            "target=$target version=$APP_VERSION " +
                "appImage=${runningAppImagePath() ?: "unavailable, self-install is not possible"}"
        )
        UpdateManager(
            client = get<HttpClient>(),
            json = UpdateModels.json,
            currentVersion = { APP_VERSION },
            target = target,
        )
    }
}
