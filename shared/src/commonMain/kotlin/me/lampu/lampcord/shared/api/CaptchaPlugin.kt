package me.lampu.lampcord.shared.api

import io.ktor.client.call.save
import io.ktor.client.plugins.HttpSend
import io.ktor.client.plugins.api.createClientPlugin
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.OutgoingContent
import me.lampu.lampcord.shared.captcha.CaptchaCoordinator
import me.lampu.lampcord.shared.utils.Logging

class CaptchaCancelledException : Exception("Captcha was cancelled")

private const val MAX_CAPTCHA_ATTEMPTS = 3

class CaptchaPluginConfig {
    lateinit var coordinator: CaptchaCoordinator
}

val LampcordCaptchaPlugin = createClientPlugin("LampcordCaptcha", ::CaptchaPluginConfig) {
    val coordinator = pluginConfig.coordinator

    client.plugin(HttpSend).intercept { builder ->
        var call = execute(builder)
        var attempts = 0
        while (call.response.status == HttpStatusCode.BadRequest) {
            call = call.save()
            val challenge = parseCaptchaChallenge(call.response.bodyAsText()) ?: break
            if (attempts >= MAX_CAPTCHA_ATTEMPTS) break
            if (builder.body is OutgoingContent.ReadChannelContent) break
            attempts++
            Logging.i("Captcha", "Solving ${challenge.service.wireName} challenge, attempt $attempts")
            val token = coordinator.solve(challenge) ?: throw CaptchaCancelledException()
            applyCaptchaHeaders(builder, token, challenge.rqtoken, challenge.sessionId)
            call = execute(builder)
        }
        call
    }
}

private fun applyCaptchaHeaders(
    builder: HttpRequestBuilder,
    token: String,
    rqtoken: String?,
    sessionId: String?,
) {
    builder.headers.remove(CAPTCHA_KEY_HEADER)
    builder.headers.remove(CAPTCHA_RQTOKEN_HEADER)
    builder.headers.remove(CAPTCHA_SESSION_HEADER)
    builder.headers.append(CAPTCHA_KEY_HEADER, token)
    rqtoken?.let { builder.headers.append(CAPTCHA_RQTOKEN_HEADER, it) }
    sessionId?.let { builder.headers.append(CAPTCHA_SESSION_HEADER, it) }
}

private const val CAPTCHA_KEY_HEADER = "X-Captcha-Key"
private const val CAPTCHA_RQTOKEN_HEADER = "X-Captcha-Rqtoken"
private const val CAPTCHA_SESSION_HEADER = "X-Captcha-Session-Id"
