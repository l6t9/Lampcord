package me.lampu.lampcord.shared.oauth

import me.lampu.lampcord.shared.utils.randomUUID
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis

internal object OAuthUrlParser {
    private const val MAX_URL_LENGTH = 2048
    private const val MAX_STATE_LENGTH = 256
    private const val MAX_SCOPE_LENGTH = 512

    private val CLIENT_ID = Regex("^[1-9][0-9]{16,19}$")
    private val CODE_CHALLENGE = Regex("^[A-Za-z0-9_-]{43}$")
    private val CUSTOM_SCHEME = Regex("^[a-z][a-z0-9+.-]*$")
    private val SNOWFLAKE = Regex("^[0-9]{1,20}$")

    private val AUTHORIZE_HOSTS = setOf(
        "discord.com",
        "www.discord.com",
        "discordapp.com",
        "www.discordapp.com",
        "canary.discord.com",
        "ptb.discord.com",
    )

    private val REJECTED_SCOPES = setOf(
        "bot",
        "webhook.incoming",
        "rpc",
        "applications.builds.upload",
        "applications.store.update",
        "applications.entitlements",
        "applications.commands.update",
        "guilds.join",
    )

    internal data class ParsedUrl(
        val scheme: String,
        val authority: String,
        val host: String,
        val path: String,
        val query: Map<String, String>,
    )

    internal fun splitUrl(url: String): ParsedUrl? {
        if (url.isEmpty() || url.length > MAX_URL_LENGTH) return null
        if (url.any { it == '\\' || it.isWhitespace() || it.isISOControl() }) return null

        val scheme: String
        val remainder: String
        val separator = url.indexOf("://")
        if (separator > 0) {
            scheme = url.substring(0, separator).lowercase()
            remainder = url.substring(separator + 3)
        } else {
            val colon = url.indexOf(':')
            if (colon <= 0) return null
            scheme = url.substring(0, colon).lowercase()
            remainder = url.substring(colon + 1)
        }
        if (!CUSTOM_SCHEME.matches(scheme)) return null

        val withoutFragment = remainder.substringBefore('#')
        if (withoutFragment.length != remainder.length) return null

        val authority: String
        val afterAuthority: String
        if (scheme == "https" || scheme == "http") {
            val end = remainder.indexOf('/').let { if (it < 0) remainder.length else it }
            authority = remainder.substring(0, end)
            afterAuthority = remainder.substring(end)
        } else {
            val end = withoutFragment.indexOf('/')
            authority = if (end >= 0) withoutFragment.substring(0, end) else withoutFragment
            afterAuthority = if (end >= 0) withoutFragment.substring(end) else ""
        }

        if (authority.contains('@')) return null
        val host = when {
            authority.startsWith("[") -> authority.substringBefore(']') + "]"
            authority.contains(':') -> authority.substringBeforeLast(':')
            else -> authority
        }.lowercase()

        val question = afterAuthority.indexOf('?')
        val path = if (question >= 0) afterAuthority.substring(0, question) else afterAuthority
        val query = linkedMapOf<String, String>()
        if (question >= 0) {
            val queryPart = afterAuthority.substring(question + 1)
            if (queryPart.isNotEmpty()) {
                for (pair in queryPart.split('&')) {
                    if (pair.isEmpty()) continue
                    val eq = pair.indexOf('=')
                    val rawKey = if (eq >= 0) pair.substring(0, eq) else pair
                    val rawValue = if (eq >= 0) pair.substring(eq + 1) else ""
                    val key = percentDecode(rawKey) ?: return null
                    val value = percentDecode(rawValue) ?: return null
                    if (query.containsKey(key)) return null
                    query[key] = value
                }
            }
        }

        return ParsedUrl(scheme, authority, host, path, query)
    }

    internal fun percentDecode(value: String): String? {
        val bytes = ArrayList<Byte>(value.length)
        var index = 0
        while (index < value.length) {
            when (val char = value[index]) {
                '+' -> {
                    bytes.add(' '.code.toByte())
                    index++
                }
                '%' -> {
                    if (index + 2 >= value.length) return null
                    val high = value[index + 1].digitToIntOrNull(16) ?: return null
                    val low = value[index + 2].digitToIntOrNull(16) ?: return null
                    bytes.add(((high shl 4) or low).toByte())
                    index += 3
                }
                else -> {
                    for (byte in char.toString().encodeToByteArray()) bytes.add(byte)
                    index++
                }
            }
        }
        return runCatching { bytes.toByteArray().decodeToString() }.getOrNull()
    }

    fun parse(url: String, source: OAuthRequestSource): OAuthAuthorizationRequest? {
        val parsed = splitUrl(url) ?: return null

        val isWeb = parsed.scheme == "https" && parsed.host in AUTHORIZE_HOSTS
        val isDiscordDeepLink = parsed.scheme == "discord" && parsed.host == "action"
        if ((!isWeb && !isDiscordDeepLink) || parsed.path != "/oauth2/authorize") return null

        val query = parsed.query
        if (query["response_type"] != "code") return null

        val clientId = query["client_id"]?.trim().orEmpty()
        if (!CLIENT_ID.matches(clientId)) return null

        val state = query["state"]?.trim()
        if (state.isNullOrEmpty() || state.length > MAX_STATE_LENGTH) return null
        if (state.any { it.isISOControl() || it.isWhitespace() }) return null

        val scopeValue = query["scope"]?.trim().orEmpty()
        if (scopeValue.isEmpty() || scopeValue.length > MAX_SCOPE_LENGTH) return null
        val scopes = scopeValue.split(' ').filter { it.isNotEmpty() }.distinct()
        if (scopes.isEmpty() || scopes.any { it in REJECTED_SCOPES }) return null

        val redirectUri = query["redirect_uri"]?.trim()?.takeIf { it.isNotEmpty() }
        if (redirectUri != null) {
            if (redirectUri.length > MAX_URL_LENGTH) return null
            if (!isPlausibleRedirect(redirectUri)) return null
        }

        val codeChallenge = query["code_challenge"]?.trim()?.takeIf { it.isNotEmpty() }
        val codeChallengeMethod = query["code_challenge_method"]?.trim()?.takeIf { it.isNotEmpty() }
        val hasPkce = codeChallenge != null
        if (codeChallenge != null && (codeChallengeMethod != "S256" || !CODE_CHALLENGE.matches(codeChallenge))) return null
        if (redirectUri != null && !isHttpRedirect(redirectUri) && !hasPkce) return null

        val guildId = query["guild_id"]?.trim()?.takeIf { SNOWFLAKE.matches(it) }
        val channelId = query["channel_id"]?.trim()?.takeIf { SNOWFLAKE.matches(it) }
        val permissions = query["permissions"]?.trim()?.takeIf { it.length <= 64 && it.all(Char::isDigit) }
        val integrationType = query["integration_type"]?.trim()?.toIntOrNull()?.takeIf { it in 0..1 }
        val nonce = query["nonce"]?.trim()?.takeIf { it.length <= MAX_STATE_LENGTH }
        val prompt = query["prompt"]?.trim()?.takeIf { it == "consent" || it == "none" }

        return OAuthAuthorizationRequest(
            id = randomUUID(),
            source = source,
            clientId = clientId,
            responseType = "code",
            scopes = scopes,
            redirectUri = redirectUri,
            state = state,
            nonce = nonce,
            codeChallenge = codeChallenge,
            codeChallengeMethod = codeChallengeMethod,
            prompt = prompt,
            integrationType = integrationType,
            permissions = permissions,
            guildId = guildId,
            channelId = channelId,
            receivedAtMillis = getCurrentTimeMillis(),
        )
    }

    internal fun isHttpRedirect(uri: String): Boolean = uri.startsWith("https://", ignoreCase = true) ||
        uri.startsWith("http://", ignoreCase = true)

    private fun isPlausibleRedirect(uri: String): Boolean {
        if (isHttpRedirect(uri)) return splitUrl(uri)?.let { it.host.isNotEmpty() } == true
        val colon = uri.indexOf(':')
        if (colon <= 0) return false
        val scheme = uri.substring(0, colon).lowercase()
        if (!CUSTOM_SCHEME.matches(scheme)) return false
        if (scheme == "javascript" || scheme == "data" || scheme == "file" || scheme == "content") return false
        return uri.substring(colon + 1).isNotBlank()
    }
}
