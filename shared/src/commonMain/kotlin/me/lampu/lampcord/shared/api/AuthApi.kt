package me.lampu.lampcord.shared.api

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import me.lampu.lampcord.shared.model.FingerprintResponse
import me.lampu.lampcord.shared.model.LoginFailure
import me.lampu.lampcord.shared.model.LoginRequest
import me.lampu.lampcord.shared.model.LoginResponse
import me.lampu.lampcord.shared.model.MFALoginRequest
import me.lampu.lampcord.shared.utils.Logging

sealed class ForgotPasswordResult {
    data object Sent : ForgotPasswordResult()
    data object PhoneVerificationRequired : ForgotPasswordResult()
    data class Failed(val message: String) : ForgotPasswordResult()
}

private const val NETWORK_ERROR_MESSAGE = "Could not reach Discord. Check your connection and try again."
private const val PASSWORD_UPDATE_MESSAGE =
    "Discord needs you to change your password before you can sign in. Update it on discord.com, then try again."

class AuthApi(
    val rest: RestClient,
) {

    fun setToken(token: String?) {
        rest.setToken(token)
    }

    suspend fun getFingerprint(): String? {
        return try {
            rest.httpClient.get("${rest.apiBase}/experiments") {
                loginHeaders(rest)
            }.body<FingerprintResponse>().fingerprint
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Gateway", "Error fetching fingerprint: ${e.message}")
            null
        }
    }

    suspend fun login(request: LoginRequest, fingerprint: String): LoginResponse? {
        return try {
            val response = postLogin(request, fingerprint)
            val body = response.bodyAsText()
            if (response.status.isSuccess()) {
                decodeLogin(body)?.let { completeLogin(it) }
            } else {
                Logging.e("Auth", "Login failed HTTP ${response.status.value}, discord code ${rest.discordError(body)?.code}")
                loginFailure(body, response.status.value)
            }
        } catch (e: CaptchaCancelledException) {
            LoginResponse(captchaCancelled = true)
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Error logging in: ${e.message}")
            networkFailure()
        }
    }

    suspend fun loginMFA(request: MFALoginRequest, fingerprint: String, type: String): LoginResponse? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/auth/mfa/$type") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(request)
            }
            val body = response.bodyAsText()
            Logging.i("Auth", "MFA ($type): verify responded HTTP ${response.status.value}")
            if (response.status.isSuccess()) {
                decodeLogin(body)?.let { completeLogin(it) }
            } else {
                Logging.e("Auth", "MFA ($type): verify failed HTTP ${response.status.value}, discord code ${rest.discordError(body)?.code}")
                LoginResponse(message = mfaFailureMessage(body, response.status.value))
            }
        } catch (e: CaptchaCancelledException) {
            LoginResponse(captchaCancelled = true)
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Error verifying MFA: ${e.message}")
            networkFailure()
        }
    }

    suspend fun sendMfaSms(ticket: String, fingerprint: String): Result<String> {
        return try {
            Logging.i("Auth", "SMS MFA: requesting code")
            val response = rest.httpClient.post("${rest.apiBase}/auth/mfa/sms/send") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("ticket", ticket) })
            }
            val body = response.bodyAsText()
            Logging.i("Auth", "SMS MFA: send responded HTTP ${response.status.value}")
            if (response.status.isSuccess()) {
                val phone = runCatching {
                    rest.json.parseToJsonElement(body).jsonObject["phone"]?.jsonPrimitive?.content
                }.getOrNull()
                if (phone.isNullOrBlank()) {
                    Logging.e("Auth", "SMS MFA: no phone field in response (body length ${body.length})")
                    Result.failure(IllegalStateException("Discord did not return a phone number."))
                } else {
                    Logging.i("Auth", "SMS MFA: code sent to number ending ${phone.takeLast(4)}")
                    Result.success(phone)
                }
            } else {
                Logging.e("Auth", "SMS MFA: send failed HTTP ${response.status.value}, discord code ${rest.discordError(body)?.code}")
                val discord = rest.discordError(body)
                val message = httpFallbackMessage(
                    status = response.status.value,
                    discordMessage = discord?.message?.takeIf { it.isNotBlank() },
                    fallback = "Could not send the SMS code (HTTP ${response.status.value})."
                )
                Result.failure(IllegalStateException(message))
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Error sending MFA SMS: ${e.message}")
            Result.failure(IllegalStateException(NETWORK_ERROR_MESSAGE))
        }
    }

    suspend fun verifyPhoneForLogin(phone: String, code: String, fingerprint: String): Result<String> {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/phone-verifications/verify") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("phone", phone); put("code", code) })
            }
            val body = response.bodyAsText()
            Logging.i("Auth", "Phone verification: verify responded HTTP ${response.status.value}")
            if (response.status.isSuccess()) {
                val token = runCatching {
                    rest.json.parseToJsonElement(body).jsonObject["token"]?.jsonPrimitive?.content
                }.getOrNull()
                if (token.isNullOrBlank()) {
                    Result.failure(IllegalStateException("Discord did not return a verification token."))
                } else {
                    Result.success(token)
                }
            } else {
                Result.failure(IllegalStateException(verificationFailureMessage(body, response.status.value)))
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Phone verification error: ${e.message}")
            Result.failure(IllegalStateException(NETWORK_ERROR_MESSAGE))
        }
    }

    suspend fun resendPhoneCode(phone: String, fingerprint: String): Result<Unit> {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/phone-verifications/resend") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("phone", phone) })
            }
            Logging.i("Auth", "Phone verification: resend responded HTTP ${response.status.value}")
            if (response.status.isSuccess()) {
                Result.success(Unit)
            } else {
                Result.failure(IllegalStateException(verificationFailureMessage(response.bodyAsText(), response.status.value)))
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Phone resend error: ${e.message}")
            Result.failure(IllegalStateException(NETWORK_ERROR_MESSAGE))
        }
    }

    suspend fun authorizeIp(token: String, fingerprint: String): Result<Unit> {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/auth/authorize-ip") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("token", token) })
            }
            if (response.status.isSuccess()) {
                Logging.i("Auth", "Login verification: location authorized")
                Result.success(Unit)
            } else {
                val body = response.bodyAsText()
                Logging.e("Auth", "Login verification: authorize-ip HTTP ${response.status.value}")
                Result.failure(IllegalStateException(verificationFailureMessage(body, response.status.value)))
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Login verification error: ${e.message}")
            Result.failure(IllegalStateException(NETWORK_ERROR_MESSAGE))
        }
    }

    private fun verificationFailureMessage(body: String, status: Int): String {
        val discord = rest.discordError(body)
        return httpFallbackMessage(
            status = status,
            discordMessage = discord?.message?.takeIf { it.isNotBlank() },
            fallback = "Could not verify this login (HTTP $status)."
        )
    }

    suspend fun forgotPassword(login: String, fingerprint: String): ForgotPasswordResult {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/auth/forgot") {
                loginHeaders(rest, fingerprint)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("login", login) })
            }
            val body = response.bodyAsText()
            Logging.i("Auth", "Forgot password: HTTP ${response.status.value}")
            val discord = rest.discordError(body)
            when {
                response.status.isSuccess() -> ForgotPasswordResult.Sent
                discord?.code == 70007 -> ForgotPasswordResult.PhoneVerificationRequired
                else -> ForgotPasswordResult.Failed(
                    httpFallbackMessage(
                        status = response.status.value,
                        discordMessage = discord?.message?.takeIf { it.isNotBlank() },
                        fallback = "Could not start a password reset (HTTP ${response.status.value})."
                    )
                )
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Forgot password error: ${e.message}")
            ForgotPasswordResult.Failed(NETWORK_ERROR_MESSAGE)
        }
    }

    suspend fun logoutRemote(token: String) {
        try {
            rest.httpClient.post("${rest.apiBase}/auth/logout") {
                loginHeaders(rest)
                header(HttpHeaders.Authorization, token)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { })
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("Auth", "Error logging out remotely: ${e.message}")
        }
    }

    suspend fun exchangeRemoteAuthTicket(ticket: String): String? {
        return try {
            val response = rest.httpClient.post("${rest.apiBase}/users/@me/remote-auth/login") {
                loginHeaders(rest)
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("ticket", ticket) })
            }
            if (response.status.isSuccess()) {
                val body = response.body<JsonObject>()
                body["encrypted_token"]?.jsonPrimitive?.content
            } else {
                null
            }
        } catch (e: CancellationException) {
            throw e
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
            } else {
                null
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            null
        }
    }

    private suspend fun postLogin(request: LoginRequest, fingerprint: String): HttpResponse =
        rest.httpClient.post("${rest.apiBase}/auth/login") {
            loginHeaders(rest, fingerprint)
            contentType(ContentType.Application.Json)
            setBody(request)
        }

    private fun decodeLogin(body: String): LoginResponse? =
        runCatching { rest.json.decodeFromString<LoginResponse>(body) }.getOrNull()

    private suspend fun completeLogin(response: LoginResponse): LoginResponse {
        val token = response.token ?: return response
        val passwordUpdateRequired = response.required_actions.orEmpty()
            .any { it.toString().contains("update_password") }
        if (!passwordUpdateRequired) return response
        logoutRemote(token)
        return LoginResponse(
            user_id = response.user_id,
            message = PASSWORD_UPDATE_MESSAGE,
            failure = LoginFailure.PASSWORD_UPDATE_REQUIRED
        )
    }

    private fun loginFailure(body: String, status: Int): LoginResponse {
        val discord = rest.discordError(body)
        val code = discord?.code
        val failure = when {
            status == 403 && body.contains("suspended_user_token") -> LoginFailure.SUSPENDED
            status == 429 -> LoginFailure.RATE_LIMITED
            status in 500..599 -> LoginFailure.SERVER_ERROR
            isCaptchaResponseBody(body) -> LoginFailure.CAPTCHA_REJECTED
            code == 70007 -> LoginFailure.VERIFICATION_REQUIRED
            code == 20013 -> LoginFailure.ACCOUNT_DISABLED
            code == 20011 -> LoginFailure.ACCOUNT_SCHEDULED_FOR_DELETION
            status == 401 || code == 50035 ||
                (status == 400 && body.contains("password", ignoreCase = true)) -> LoginFailure.INVALID_CREDENTIALS
            else -> LoginFailure.UNKNOWN
        }
        val message = when (failure) {
            LoginFailure.SUSPENDED -> "This account has been suspended by Discord. You can appeal the suspension on discord.com."
            LoginFailure.RATE_LIMITED -> "Discord is rate limiting login attempts. Wait a moment and try again."
            LoginFailure.SERVER_ERROR -> "Discord is having trouble right now (HTTP $status). Try again in a moment."
            LoginFailure.CAPTCHA_REJECTED -> "Discord rejected the captcha. Try again."
            LoginFailure.VERIFICATION_REQUIRED ->
                "Discord needs to verify this login from a new location."
            LoginFailure.ACCOUNT_DISABLED -> "This account is disabled. You can restore it to sign in."
            LoginFailure.ACCOUNT_SCHEDULED_FOR_DELETION -> "This account is scheduled for deletion. You can restore it to sign in."
            LoginFailure.INVALID_CREDENTIALS -> "Invalid email or password."
            else -> httpFallbackMessage(status, discord?.message?.takeIf { it.isNotBlank() }, "Login failed (HTTP $status).")
        }
        return LoginResponse(code = code, message = message, failure = failure)
    }

    private fun mfaFailureMessage(body: String, status: Int): String {
        val discord = rest.discordError(body)
        return when {
            status == 401 || discord?.code == 50035 -> "That code is invalid or has expired."
            isCaptchaResponseBody(body) -> "Discord rejected the captcha. Try again."
            else -> httpFallbackMessage(
                status = status,
                discordMessage = discord?.message?.takeIf { it.isNotBlank() },
                fallback = "Could not verify that code (HTTP $status)."
            )
        }
    }

    private fun httpFallbackMessage(status: Int, discordMessage: String?, fallback: String): String = when {
        status == 429 -> "Discord is rate limiting requests. Wait a moment and try again."
        status in 500..599 -> "Discord is having trouble right now (HTTP $status). Try again in a moment."
        discordMessage != null -> discordMessage
        else -> fallback
    }

    private fun networkFailure(): LoginResponse =
        LoginResponse(message = NETWORK_ERROR_MESSAGE, failure = LoginFailure.NETWORK)
}
