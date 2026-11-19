package me.lampu.lampcord.shared.oauth

import kotlinx.serialization.Serializable

enum class OAuthRequestSource {
    ANDROID_DEEPLINK,
    DESKTOP_URL,
    DESKTOP_RPC,
}

data class OAuthAuthorizationRequest(
    val id: String,
    val source: OAuthRequestSource,
    val clientId: String,
    val responseType: String,
    val scopes: List<String>,
    val redirectUri: String?,
    val state: String?,
    val nonce: String?,
    val codeChallenge: String?,
    val codeChallengeMethod: String?,
    val prompt: String?,
    val integrationType: Int?,
    val permissions: String?,
    val guildId: String?,
    val channelId: String?,
    val receivedAtMillis: Long,
)

sealed interface OAuthResult {
    val requestId: String
    val redirectUrl: String?

    data class Success(
        override val requestId: String,
        val code: String,
        override val redirectUrl: String,
    ) : OAuthResult

    data class Denied(
        override val requestId: String,
        val error: String = "access_denied",
        override val redirectUrl: String? = null,
    ) : OAuthResult

    data class Failed(
        override val requestId: String,
        val error: String,
        val message: String? = null,
        override val redirectUrl: String? = null,
    ) : OAuthResult
}

@Serializable
data class OAuthAuthorizePreview(
    val application: OAuthPreviewApplication? = null,
    val authorized: Boolean = false,
    val integration_type: Int? = null,
    val redirect_uri: String? = null,
)

@Serializable
data class OAuthPreviewApplication(
    val id: String,
    val name: String,
    val icon: String? = null,
    val description: String? = null,
)

@Serializable
data class OAuthAuthorizeResponse(
    val url: String? = null,
)
