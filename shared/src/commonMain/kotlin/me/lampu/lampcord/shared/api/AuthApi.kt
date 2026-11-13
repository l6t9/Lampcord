package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import io.ktor.client.statement.HttpResponse
import me.lampu.lampcord.shared.model.FingerprintResponse
import me.lampu.lampcord.shared.model.LoginRequest
import me.lampu.lampcord.shared.model.LoginResponse
import me.lampu.lampcord.shared.model.MFALoginRequest
import me.lampu.lampcord.shared.utils.Logging

class AuthApi(
    val rest: RestClient,
) {

    fun setToken(token: String?) {
        rest.setToken(token)
    }

    suspend fun getFingerprint(): String? {
        return try {
            val response: FingerprintResponse = rest.httpClient.get("${rest.apiBase}/experiments") {
                loginHeaders(rest)
            }.body()
            response.fingerprint
        } catch (e: Exception) {
            Logging.e("Gateway", "Error fetching fingerprint: ${e.message}")
            null
        }
    }

    suspend fun login(request: LoginRequest, fingerprint: String): LoginResponse? {
        return try {
            val response = postLogin(request, fingerprint)
            val responseBody = response.bodyAsText()
            if (response.status.isSuccess()) {
                runCatching { rest.json.decodeFromString<LoginResponse>(responseBody) }.getOrNull()
            } else {
                val message = loginErrorMessage(rest, responseBody, response.status.value)
                Logging.e("Auth", "Login failed with status: ${response.status}, captchaRejected=${isCaptchaResponseBody(responseBody)}")
                LoginResponse(message = message)
            }
        } catch (e: CaptchaCancelledException) {
            LoginResponse(captchaCancelled = true)
        } catch (e: Exception) {
            Logging.e("Auth", "Error logging in: ${e.message}")
            LoginResponse(message = "Could not reach Discord. Check your connection and try again.")
        }
    }

    private suspend fun postLogin(request: LoginRequest, fingerprint: String): HttpResponse =
        rest.httpClient.post("${rest.apiBase}/auth/login") {
            loginHeaders(rest, fingerprint)
            contentType(ContentType.Application.Json)
            setBody(request)
        }

    suspend fun loginMFA(request: MFALoginRequest, fingerprint: String, type: String): LoginResponse? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/auth/mfa/$type") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            val responseBody = response.bodyAsText()
            if (response.status.isSuccess()) {
                runCatching { rest.json.decodeFromString<LoginResponse>(responseBody) }.getOrNull()
            } else {
                Logging.e("Auth", "MFA failed with status: ${response.status}, body: $responseBody")
                LoginResponse(message = mfaErrorMessage(rest, responseBody, response.status.value))
            }
        } catch (e: CaptchaCancelledException) {
            LoginResponse(captchaCancelled = true)
        } catch (e: Exception) {
            Logging.e("Auth", "Error verifying MFA: ${e.message}")
            LoginResponse(message = "Could not reach Discord. Check your connection and try again.")
        }
    }

    private fun loginErrorMessage(rest: RestClient, body: String, status: Int): String {
        val discord = rest.discordError(body)
        return when {
            status == 401 -> "Invalid email or password."
            status == 400 && body.contains("password", ignoreCase = true) -> "Invalid email or password."
            status == 429 -> "Discord is rate limiting login attempts. Wait a moment and try again."
            status in 500..599 -> "Discord is having trouble right now (HTTP $status). Try again in a moment."
            isCaptchaResponseBody(body) -> "Discord rejected the captcha. Try again."
            discord?.code == 50035 -> "Invalid email or password."
            !discord?.message.isNullOrBlank() -> discord.message
            else -> "Login failed (HTTP $status)."
        }
    }

    private fun mfaErrorMessage(rest: RestClient, body: String, status: Int): String {
        val discord = rest.discordError(body)
        return when {
            status == 401 -> "That code is invalid or has expired."
            status == 429 -> "Discord is rate limiting login attempts. Wait a moment and try again."
            status in 500..599 -> "Discord is having trouble right now (HTTP $status). Try again in a moment."
            isCaptchaResponseBody(body) -> "Discord rejected the captcha. Try again."
            discord?.code == 50035 -> "That code is invalid or has expired."
            !discord?.message.isNullOrBlank() -> discord.message
            else -> "Could not verify that code (HTTP $status)."
        }
    }

    suspend fun exchangeRemoteAuthTicket(ticket: String): String? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/users/@me/remote-auth/login") {
                loginHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(kotlinx.serialization.json.buildJsonObject { put("ticket", ticket) })
            }
            if (response.status.isSuccess()) {
                val body = response.body<JsonObject>()
                body["encrypted_token"]?.jsonPrimitive?.content
            } else {
                null
            }
        } catch (e: Exception) {
            Logging.e("Auth", "Error exchanging remote auth ticket: ${e.message}")
            null
        }
    }

    suspend fun getGatewayUrl(): String? {
        return try {
            val response = rest.httpClient.get("${rest.apiBase}/gateway") {
                standardHeaders(rest)
            }
            if (response.status.isSuccess()) {
                val body = response.body<JsonObject>()
                body["url"]?.jsonPrimitive?.content
            } else null
        } catch (e: Exception) { null }
    }
}
