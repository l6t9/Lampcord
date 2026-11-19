package me.lampu.lampcord.shared.api

import kotlinx.datetime.TimeZone
import kotlin.random.Random
import me.lampu.lampcord.shared.settings.Settings
import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.logging.LogLevel
import io.ktor.client.plugins.logging.Logging
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.cookies.HttpCookies
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.header
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.HttpReceivePipeline
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import me.lampu.lampcord.shared.captcha.CaptchaCoordinator
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import me.lampu.lampcord.shared.utils.getDeviceName
import me.lampu.lampcord.shared.utils.getOsVersion
import me.lampu.lampcord.shared.utils.getPlatformName
import me.lampu.lampcord.shared.utils.randomUUID
import me.lampu.lampcord.shared.utils.Logging as SharedLogging
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

class RestClient(
    val httpClient: HttpClient,
    val json: Json
) {
    var token: String? = null
        private set

    fun closeClient() {
        httpClient.close()
    }

    val unauthorizedEvents = MutableSharedFlow<String>()

    init {
        httpClient.receivePipeline.intercept(HttpReceivePipeline.Before) { response ->
            // Only an authenticated request that was rejected means this session is dead.
            // A 401 from a login/MFA request must not log out the account already in use.
            // Emit the rejected token so a stale request cannot log out a newer session.
            val authHeader = response.call.request.headers[HttpHeaders.Authorization]
            if (response.status == HttpStatusCode.Unauthorized && authHeader != null) {
                unauthorizedEvents.emit(authHeader)
            }
            proceedWith(response)
        }
    }

    private val apiVersion = 9
    val apiBase = "https://discord.com/api/v$apiVersion"

    val vendorId = randomUUID()
    val launchId = randomUUID()
    val heartbeatSessionId = randomUUID()
    val launchSignature = (getCurrentTimeMillis() * 1_000_000L).toString()

    fun setToken(token: String?) {
        this.token = token
    }

    private val cachedSuperProperties: String by lazy { buildSuperProperties() }

    fun getSuperProperties(): String = cachedSuperProperties

    @OptIn(ExperimentalEncodingApi::class)
    private fun buildSuperProperties(): String {
        val platform = getPlatformName()
        val isMobile = platform == "android" || platform == "ios"

        val properties = buildJsonObject {
            put("os", if (platform == "macos") "Mac OS X" else platform.replaceFirstChar { it.uppercase() })
            put("browser", if (isMobile) "Discord Android" else "Discord Client")
            put("device", getDeviceName())
            put("system_locale", "en-US")
            put("has_client_mods", false)
            put("client_version", if (isMobile) "341.0 - rn" else "0.0.398")
            put("release_channel", if (isMobile) "canaryRelease" else "stable")
            put("device_vendor_id", vendorId)
            put("design_id", 2)
            put("browser_user_agent", if (isMobile) {
                if (platform == "android") "Discord-Android/341200;RNA" else "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
            } else "")
            put("browser_version", if (isMobile) "" else "37.6.0")
            put("os_version", getOsVersion())
            put("client_build_number", if (isMobile) 6081 else 575562)
            put("client_event_source", JsonNull)
            if (isMobile) {
                put("client_launch_id", launchId)
                put("launch_signature", launchSignature)
                put("client_app_state", "active")
                put("client_heartbeat_session_id", heartbeatSessionId)
            } else {
                put("client_launch_id", JsonNull)
                put("launch_signature", "")
                put("client_app_state", JsonNull)
                put("client_heartbeat_session_id", JsonNull)
            }
        }
        val jsonString = json.encodeToString(properties)
        return Base64.encode(jsonString.encodeToByteArray())
    }
}

@Serializable
data class DiscordApiError(val code: Int? = null, val message: String? = null)

fun RestClient.discordError(body: String): DiscordApiError? =
    runCatching { json.decodeFromString<DiscordApiError>(body) }.getOrNull()

fun HttpRequestBuilder.standardHeaders(rest: RestClient) {
    val platform = getPlatformName()
    val isMobile = platform == "android" || platform == "ios"

    val userAgent = if (isMobile) {
        if (platform == "android") {
            "Discord-Android/341200;RNA"
        } else {
            "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
        }
    } else {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
    }

    header("User-Agent", userAgent)
    header("Accept-Language", "en-US,en;q=0.9")
    header("X-Super-Properties", rest.getSuperProperties())
    header("X-Discord-Locale", "en-US")
    rest.token?.let { header(HttpHeaders.Authorization, it) }
}

fun HttpRequestBuilder.loginHeaders(rest: RestClient, fingerprint: String? = null) {
    val platform = getPlatformName()
    val isMobile = platform == "android" || platform == "ios"
    val userAgent = if (isMobile) {
        if (platform == "android") "Discord-Android/341200;RNA" else "Discord/105180 CFNetwork/1410.0.3 Darwin/22.4.0"
    } else {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36"
    }

    header("User-Agent", userAgent)
    header("Accept-Language", "en-US,en;q=0.9")
    header("X-Super-Properties", rest.getSuperProperties())
    header("X-Discord-Locale", "en-US")
    header("X-Discord-Timezone", TimeZone.currentSystemDefault().id)
    header("X-Installation-Id", installationId())
    fingerprint?.let { header("X-Fingerprint", it) }
    header("Origin", "https://discord.com")
    header("Referer", "https://discord.com/login")
}

private var cachedInstallationId: String? = null

private fun installationId(): String {
    cachedInstallationId?.let { return it }
    val saved = Settings.shared.installationId
    val id = if (saved.isNotBlank()) {
        saved
    } else {
        val hex = (1..32).joinToString("") { Random.nextInt(16).toString(16) }
        val created = "${hex.substring(0, 8)}-${hex.substring(8, 12)}-${hex.substring(12, 16)}-${hex.substring(16, 20)}-${hex.substring(20)}"
        Settings.shared.installationId = created
        created
    }
    cachedInstallationId = id
    return id
}

fun createHttpClient(captcha: CaptchaCoordinator) = HttpClient(httpClientEngine()) {
    install(HttpCookies)
    install(LampcordCaptchaPlugin) {
        coordinator = captcha
    }
    install(HttpTimeout) {
        requestTimeoutMillis = 15000
        connectTimeoutMillis = 10000
        socketTimeoutMillis = 15000
    }
    install(Logging) {
        logger = SharedLogging.ktorLogger
        level = if (SharedLogging.debugEnabled) LogLevel.INFO else LogLevel.NONE
    }
    install(ContentNegotiation) {
        json(Json {
            ignoreUnknownKeys = true
            coerceInputValues = true
        })
    }
    install(WebSockets)

    expectSuccess = false
}
