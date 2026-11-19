package me.lampu.lampcord.shared.oauth

internal object OAuthRedirectPolicy {
    private const val ISSUER = "https://discord.com"
    private val LOOPBACK_HOSTS = setOf("127.0.0.1", "localhost", "::1", "[::1]")

    internal data class Validation(
        val code: String? = null,
        val error: String? = null,
        val errorDescription: String? = null,
        val redirectUrl: String? = null,
    )

    internal fun validate(url: String, request: OAuthAuthorizationRequest): Validation {
        val parsed = OAuthUrlParser.splitUrl(url) ?: return Validation(error = "invalid_redirect")
        if (parsed.authority.contains('@')) return Validation(error = "invalid_redirect")

        val scheme = parsed.scheme
        val isHttps = scheme == "https"
        val isLoopbackHttp = scheme == "http" && parsed.host in LOOPBACK_HOSTS
        val isCustom = !isHttps && !isLoopbackHttp
        if (isCustom && parsed.path != "/authorize/callback") {
            return Validation(error = "invalid_redirect")
        }

        if (request.redirectUri != null && !matchesRequestedRedirect(parsed, request.redirectUri)) {
            return Validation(error = "redirect_mismatch")
        }

        val query = parsed.query
        query["error"]?.let { error ->
            return Validation(
                error = error,
                errorDescription = query["error_description"],
                redirectUrl = url,
            )
        }

        val state = query["state"]
        if (request.state != null && state != request.state) {
            return Validation(error = "invalid_state")
        }
        query["iss"]?.let { issuer ->
            if (issuer.trimEnd('/') != ISSUER) return Validation(error = "invalid_issuer")
        }

        val code = query["code"]?.trim()
        if (code.isNullOrEmpty() || code.length > 512) {
            return Validation(error = "missing_code", redirectUrl = url)
        }
        return Validation(code = code, redirectUrl = url)
    }

    private fun matchesRequestedRedirect(returned: OAuthUrlParser.ParsedUrl, requested: String): Boolean {
        val expected = OAuthUrlParser.splitUrl(requested) ?: return false
        if (expected.scheme != returned.scheme) return false
        if (expected.scheme == "https" || expected.scheme == "http") {
            return expected.host == returned.host && normalizePath(expected.path) == normalizePath(returned.path)
        }
        return normalizePath(expected.path) == normalizePath(returned.path)
    }

    private fun normalizePath(path: String): String = path.trim().trimEnd('/').ifEmpty { "/" }
}
