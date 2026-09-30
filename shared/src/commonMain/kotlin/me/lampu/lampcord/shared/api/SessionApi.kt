package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.request.request
import io.ktor.http.ContentType
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.buildJsonArray

data class DeviceSession(
    val idHash: String,
    val name: String,
    val location: String?,
    val lastUsed: String?,
    val current: Boolean,
    val icon: DeviceIcon
)

enum class DeviceIcon { Mobile, Desktop, Unknown }

class SessionFailure(val status: Int, val body: JsonObject?) : Exception(
    body?.let { it.message() } ?: "HTTP $status"
) {
    val mfa: JsonObject? get() = body?.get("mfa") as? JsonObject
    val requiresPassword: Boolean
        get() = (body?.containsKey("password") == true) ||
            (message.orEmpty().contains("password", ignoreCase = true))
    val requiresCode: Boolean
        get() = (body?.containsKey("code") == true) ||
            message.orEmpty().contains("two-factor", ignoreCase = true) ||
            message.orEmpty().contains("2fa", ignoreCase = true)
}

private fun JsonObject.message(): String? {
    val raw = this["message"] ?: this["error"] ?: return null
    val primitive = raw as? JsonPrimitive ?: return null
    if (!primitive.isString) return null
    return primitive.content.takeIf { it.isNotBlank() && it != "null" }
}

class SessionApi(private val rest: RestClient) {

    private val json = Json { ignoreUnknownKeys = true }

    suspend fun listSessions(): List<DeviceSession> {
        val response = request("/auth/sessions", HttpMethod.Get)
        val items = response["user_sessions"] as? JsonArray
            ?: response["sessions"] as? JsonArray
            ?: error("Discord returned an unrecognized Devices response")

        val currentHash = response["current_session_id_hash"].text()
            ?: response["auth_session_id_hash"].text()

        val sessions = items.mapNotNull { element ->
            val item = element as? JsonObject ?: return@mapNotNull null
            val hash = item["id_hash"].text()
                ?: item["session_id_hash"].text()
                ?: item["id"].text()
                ?: return@mapNotNull null

            val info = item["client_info"] as? JsonObject
            val os = info?.get("os").text()
            val platform = info?.get("platform").text()
            val browser = info?.get("browser").text()
            val device = info?.get("device").text()
            val name = listOfNotNull(device, browser, os, platform)
                .distinct()
                .take(2)
                .joinToString(" · ")
                .ifEmpty { "Unknown device" }

            val location = info?.get("location").text() ?: item["location"].text()
            val lastUsed = item["approx_last_used_time"].text() ?: item["last_used"].text()
            val isCurrent = (item["is_current"] as? JsonPrimitive)?.content?.toBooleanStrictOrNull() == true ||
                (item["current"] as? JsonPrimitive)?.content?.toBooleanStrictOrNull() == true ||
                (currentHash != null && hash == currentHash)

            DeviceSession(
                idHash = hash,
                name = name,
                location = location,
                lastUsed = lastUsed,
                current = isCurrent,
                icon = deviceIcon(name)
            )
        }
        if (items.isNotEmpty() && sessions.isEmpty()) {
            error("Discord returned sessions in an unrecognized format")
        }
        return sessions
    }

    suspend fun logout(
        idHashes: List<String>,
        password: String? = null,
        mfaToken: String? = null,
        backupCode: String? = null
    ) {
        val payload = buildJsonObject {
            put("session_id_hashes", buildJsonArray { idHashes.forEach { add(it) } })
            password?.takeIf { it.isNotEmpty() }?.let { put("password", it) }
            backupCode?.takeIf { it.isNotEmpty() }?.let { put("code", it) }
        }
        request("/auth/sessions/logout", HttpMethod.Post, payload, mfaToken)
    }

    suspend fun finishMfa(ticket: String, type: String, code: String): String {
        val payload = buildJsonObject {
            put("ticket", ticket)
            put("mfa_type", type)
            put("data", code)
        }
        val response = request("/mfa/finish", HttpMethod.Post, payload)
        return response["token"].text() ?: error("Discord did not return an MFA authorization token")
    }

    suspend fun request(
        route: String,
        method: HttpMethod,
        body: kotlinx.serialization.json.JsonObject? = null,
        mfaToken: String? = null
    ): JsonObject {
        val response = rest.httpClient.request("${rest.apiBase}$route") {
            this.method = method
            standardHeaders(rest)
            contentType(ContentType.Application.Json)
            mfaToken?.let { header("X-Discord-MFA-Authorization", it) }
            body?.let { setBody(it) }
        }
        if (!response.status.isSuccess()) {
            val errorBody = runCatching {
                json.parseToJsonElement(response.body<String>()) as? JsonObject
            }.getOrNull()
            throw SessionFailure(response.status.value, errorBody)
        }
        val text = response.body<String>()
        if (text.isBlank()) return JsonObject(emptyMap())
        return json.parseToJsonElement(text) as? JsonObject ?: JsonObject(emptyMap())
    }
}

private fun JsonElement?.text(): String? {
    val primitive = this as? JsonPrimitive ?: return null
    return primitive.takeIf { it.isString }?.content?.takeIf { it.isNotBlank() && it != "null" }
}

private fun deviceIcon(name: String): DeviceIcon {
    val mobile = listOf("android", "ios", "iphone", "ipad", "mobile", "pixel", "samsung")
        .any { name.contains(it, ignoreCase = true) }
    return when {
        mobile -> DeviceIcon.Mobile
        name.isNotBlank() -> DeviceIcon.Desktop
        else -> DeviceIcon.Unknown
    }
}