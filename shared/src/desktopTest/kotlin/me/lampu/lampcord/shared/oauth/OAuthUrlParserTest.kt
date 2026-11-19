package me.lampu.lampcord.shared.oauth

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class OAuthUrlParserTest {
    private val baseUrl = "https://discord.com/oauth2/authorize"

    private fun request(
        redirectUri: String? = "http://127.0.0.1:12345/callback",
        state: String? = "state-1234567890",
        codeChallenge: String? = null,
        codeChallengeMethod: String? = null,
    ) = OAuthAuthorizationRequest(
        id = "request-1",
        source = OAuthRequestSource.ANDROID_DEEPLINK,
        clientId = "157730590492196864",
        responseType = "code",
        scopes = listOf("openid", "sdk.social_layer_presence"),
        redirectUri = redirectUri,
        state = state,
        nonce = null,
        codeChallenge = codeChallenge,
        codeChallengeMethod = codeChallengeMethod,
        prompt = "consent",
        integrationType = null,
        permissions = null,
        guildId = null,
        channelId = null,
        receivedAtMillis = 0L,
    )

    @Test
    fun `parses documented web authorization url`() {
        val request = OAuthUrlParser.parse(
            "$baseUrl?response_type=code&client_id=157730590492196864" +
                "&scope=openid%20sdk.social_layer_presence" +
                "&redirect_uri=http%3A%2F%2F127.0.0.1%3A12345%2Fcallback" +
                "&state=state-1234567890&prompt=consent",
            OAuthRequestSource.ANDROID_DEEPLINK,
        )

        assertNotNull(request)
        assertEquals("157730590492196864", request.clientId)
        assertEquals(listOf("openid", "sdk.social_layer_presence"), request.scopes)
        assertEquals("http://127.0.0.1:12345/callback", request.redirectUri)
        assertEquals("state-1234567890", request.state)
    }

    @Test
    fun `parses discord deep link authorization url`() {
        val request = OAuthUrlParser.parse(
            "discord://action/oauth2/authorize?response_type=code&client_id=157730590492196864" +
                "&scope=sdk.social_layer&state=state-1234567890" +
                "&redirect_uri=discord-157730590492196864%3A%2Fauthorize%2Fcallback" +
                "&code_challenge=${"a".repeat(43)}&code_challenge_method=S256",
            OAuthRequestSource.ANDROID_DEEPLINK,
        )

        assertNotNull(request)
        assertEquals(OAuthRequestSource.ANDROID_DEEPLINK, request.source)
        assertEquals("S256", request.codeChallengeMethod)
    }

    @Test
    fun `rejects unsafe or malformed requests`() {
        assertNull(
            OAuthUrlParser.parse(
                "$baseUrl?response_type=token&client_id=157730590492196864&scope=identify&state=state-1234567890",
                OAuthRequestSource.DESKTOP_URL,
            )
        )
        assertNull(
            OAuthUrlParser.parse(
                "$baseUrl?response_type=code&client_id=123&scope=identify&state=state-1234567890",
                OAuthRequestSource.DESKTOP_URL,
            )
        )
        assertNull(
            OAuthUrlParser.parse(
                "$baseUrl?response_type=code&client_id=157730590492196864&scope=bot&state=state-1234567890",
                OAuthRequestSource.DESKTOP_URL,
            )
        )
        assertNull(
            OAuthUrlParser.parse(
                "https://evil.example/oauth2/authorize?response_type=code&client_id=157730590492196864&scope=identify&state=state-1234567890",
                OAuthRequestSource.DESKTOP_URL,
            )
        )
        assertNull(
            OAuthUrlParser.parse(
                "$baseUrl?response_type=code&client_id=157730590492196864&scope=identify&state=state-1234567890" +
                    "&redirect_uri=mygame%3A%2Fauthorize%2Fcallback",
                OAuthRequestSource.DESKTOP_URL,
            )
        )
    }

    @Test
    fun `validates discord issued redirect and rejects tampering`() {
        val successful = OAuthRedirectPolicy.validate(
            "http://127.0.0.1:12345/callback?code=abc123&state=state-1234567890&iss=https%3A%2F%2Fdiscord.com",
            request(),
        )
        assertEquals("abc123", successful.code)
        assertNull(successful.error)

        val wrongState = OAuthRedirectPolicy.validate(
            "http://127.0.0.1:12345/callback?code=abc123&state=other&iss=https%3A%2F%2Fdiscord.com",
            request(),
        )
        assertEquals("invalid_state", wrongState.error)

        val denied = OAuthRedirectPolicy.validate(
            "http://127.0.0.1:12345/callback?error=access_denied&state=state-1234567890",
            request(),
        )
        assertEquals("access_denied", denied.error)

        val remoteHttp = OAuthRedirectPolicy.validate(
            "http://evil.example/callback?code=abc123&state=state-1234567890",
            request(redirectUri = "http://evil.example/callback"),
        )
        assertEquals("invalid_redirect", remoteHttp.error)
    }

    @Test
    fun `custom scheme redirects require authorize callback path`() {
        val custom = "discord-157730590492196864:/authorize/callback"
        val valid = OAuthRedirectPolicy.validate(
            "$custom?code=abc123&state=state-1234567890",
            request(redirectUri = custom, codeChallenge = "a".repeat(43), codeChallengeMethod = "S256"),
        )
        assertEquals("abc123", valid.code)

        val invalid = OAuthRedirectPolicy.validate(
            "discord-157730590492196864:/evil?code=abc123&state=state-1234567890",
            request(redirectUri = "discord-157730590492196864:/evil", codeChallenge = "a".repeat(43), codeChallengeMethod = "S256"),
        )
        assertEquals("invalid_redirect", invalid.error)
    }
}
