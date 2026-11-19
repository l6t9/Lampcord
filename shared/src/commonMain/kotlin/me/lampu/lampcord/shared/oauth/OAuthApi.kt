package me.lampu.lampcord.shared.oauth

import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.parameter
import io.ktor.client.request.post
import io.ktor.client.request.get
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import me.lampu.lampcord.shared.api.RestClient
import me.lampu.lampcord.shared.api.standardHeaders
import me.lampu.lampcord.shared.utils.Logging

class OAuthApi(private val rest: RestClient) {
    suspend fun preview(request: OAuthAuthorizationRequest): Result<OAuthAuthorizePreview> = try {
        val response = rest.httpClient.get("${rest.apiBase}/oauth2/authorize") {
            standardHeaders(rest)
            applyQuery(this, request)
            timeout { requestTimeoutMillis = 15_000 }
        }
        if (!response.status.isSuccess()) {
            throw IllegalStateException("Preview failed (${response.status.value})")
        }
        Result.success(response.body<OAuthAuthorizePreview>())
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Logging.e(TAG, "OAuth preview failed: ${e.message}")
        Result.failure(e)
    }

    suspend fun authorize(request: OAuthAuthorizationRequest, approve: Boolean): Result<String> = try {
        val response = rest.httpClient.post("${rest.apiBase}/oauth2/authorize") {
            standardHeaders(rest)
            applyQuery(this, request)
            contentType(ContentType.Application.Json)
            setBody(
                buildJsonObject {
                    put("authorize", approve)
                    request.integrationType?.let { put("integration_type", it) }
                }
            )
            timeout { requestTimeoutMillis = 15_000 }
        }
        if (!response.status.isSuccess()) {
            throw IllegalStateException("Authorize failed (${response.status.value})")
        }
        val url = response.body<OAuthAuthorizeResponse>().url
        if (url.isNullOrBlank()) throw IllegalStateException("Authorize returned no redirect URL")
        Result.success(url)
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Logging.e(TAG, "OAuth authorize failed: ${e.message}")
        Result.failure(e)
    }

    private fun applyQuery(builder: HttpRequestBuilder, request: OAuthAuthorizationRequest) {
        builder.parameter("client_id", request.clientId)
        builder.parameter("response_type", request.responseType)
        builder.parameter("scope", request.scopes.joinToString(" "))
        request.redirectUri?.let { builder.parameter("redirect_uri", it) }
        request.state?.let { builder.parameter("state", it) }
        request.nonce?.let { builder.parameter("nonce", it) }
        request.codeChallenge?.let { builder.parameter("code_challenge", it) }
        request.codeChallengeMethod?.let { builder.parameter("code_challenge_method", it) }
        request.integrationType?.let { builder.parameter("integration_type", it) }
    }

    private companion object {
        const val TAG = "OAuth"
    }
}
