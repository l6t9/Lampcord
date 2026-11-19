package me.lampu.lampcord.shared.oauth

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ApplicationApi
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis

data class OAuthUiState(
    val request: OAuthAuthorizationRequest? = null,
    val appName: String? = null,
    val appIconUrl: String? = null,
    val appDescription: String? = null,
    val redirectUri: String? = null,
    val isLoading: Boolean = false,
    val error: String? = null,
)

class OAuthAuthorizationHandler(
    private val oauthApi: OAuthApi,
    private val applicationApi: ApplicationApi,
    private val scope: CoroutineScope,
) {
    private val _state = MutableStateFlow(OAuthUiState())
    val state: StateFlow<OAuthUiState> = _state.asStateFlow()

    private val _results = MutableSharedFlow<OAuthResult>(extraBufferCapacity = 4)
    val results: SharedFlow<OAuthResult> = _results.asSharedFlow()

    var openRedirect: (String) -> Unit = { }

    private var pending: OAuthAuthorizationRequest? = null

    fun handleIncomingUrl(url: String, source: OAuthRequestSource): Boolean {
        val request = OAuthUrlParser.parse(url, source) ?: return false
        return beginAuthorization(request)
    }

    fun beginAuthorization(request: OAuthAuthorizationRequest): Boolean {
        if (getCurrentTimeMillis() - request.receivedAtMillis > REQUEST_TTL_MS) return false

        pending = request
        _state.value = OAuthUiState(request = request, redirectUri = request.redirectUri, isLoading = true)
        scope.launch {
            val app = applicationApi.getApplications(listOf(request.clientId)).firstOrNull()
            val preview = oauthApi.preview(request).getOrNull()
            val name = preview?.application?.name ?: app?.name ?: request.clientId
            val iconHash = preview?.application?.icon ?: app?.icon
            val iconUrl = if (iconHash.isNullOrBlank()) {
                null
            } else {
                "https://cdn.discordapp.com/app-icons/${request.clientId}/$iconHash.png?size=64"
            }
            if (pending?.id != request.id) return@launch
            _state.value = OAuthUiState(
                request = request,
                appName = name,
                appIconUrl = iconUrl,
                appDescription = preview?.application?.description ?: app?.description,
                redirectUri = preview?.redirect_uri ?: request.redirectUri,
                isLoading = false,
            )
        }
        return true
    }

    fun approve() = submit(approve = true)

    fun deny() = submit(approve = false)

    fun dismiss() {
        pending = null
        _state.value = OAuthUiState()
    }

    private fun submit(approve: Boolean) {
        val request = pending ?: return
        pending = null
        _state.value = _state.value.copy(isLoading = true, error = null)

        scope.launch {
            val authorizeResult = oauthApi.authorize(request, approve)
            if (authorizeResult.isFailure) {
                val error = authorizeResult.exceptionOrNull()
                _state.value = _state.value.copy(
                    isLoading = false,
                    error = error?.message ?: "oauth_request_failed",
                )
                _results.emit(OAuthResult.Failed(request.id, "oauth_request_failed", error?.message))
                return@launch
            }
            val url = authorizeResult.getOrThrow()

            val validation = OAuthRedirectPolicy.validate(url, request)
            val errorCode = validation.error
            val code = validation.code
            val result = when {
                errorCode == "access_denied" -> OAuthResult.Denied(
                    requestId = request.id,
                    error = "access_denied",
                    redirectUrl = validation.redirectUrl,
                )
                errorCode != null -> OAuthResult.Failed(
                    requestId = request.id,
                    error = errorCode,
                    message = validation.errorDescription,
                    redirectUrl = validation.redirectUrl,
                )
                code != null -> OAuthResult.Success(request.id, code, url)
                else -> OAuthResult.Failed(request.id, "invalid_redirect", redirectUrl = validation.redirectUrl)
            }

            _state.value = OAuthUiState()
            _results.emit(result)

            val redirect = result.redirectUrl
            if (redirect != null && request.source != OAuthRequestSource.DESKTOP_RPC) {
                runCatching { openRedirect(redirect) }
                    .onFailure { Logging.e(TAG, "Failed to deliver authorization redirect", it) }
            }
        }
    }

    private companion object {
        const val TAG = "OAuth"
        const val REQUEST_TTL_MS = 5 * 60 * 1000L
    }
}
