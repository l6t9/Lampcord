package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
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
import me.lampu.lampcord.shared.model.FingerprintResponse
import me.lampu.lampcord.shared.model.LoginRequest
import me.lampu.lampcord.shared.model.LoginResponse
import me.lampu.lampcord.shared.model.MFALoginRequest
import me.lampu.lampcord.shared.utils.Logging

class AuthApi(val rest: RestClient) {

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
            val response = rest.httpClient.post("${rest.apiBase}/auth/login") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(request)
            }

            val responseBody = response.bodyAsText()
            if (response.status.isSuccess() || response.status.value == 400 || response.status.value == 401) {
                rest.json.decodeFromString<LoginResponse>(responseBody)
            } else {
                Logging.e("Auth", "Login failed with status: ${response.status}, body: $responseBody")
                null
            }
        } catch (e: Exception) {
            Logging.e("Auth", "Error logging in: ${e.message}")
            null
        }
    }

    suspend fun loginMFA(request: MFALoginRequest, fingerprint: String, type: String): LoginResponse? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/auth/mfa/$type") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            if (response.status.isSuccess()) {
                response.body<LoginResponse>()
            } else {
                val errorBody = response.bodyAsText()
                Logging.e("Auth", "MFA failed with status: ${response.status}, body: $errorBody")
                null
            }
        } catch (e: Exception) {
            Logging.e("Auth", "Error verifying MFA: ${e.message}")
            null
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
