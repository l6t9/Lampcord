package me.lampu.lampcord.shared.api

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import me.lampu.lampcord.shared.captcha.CaptchaCoordinator
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

private const val CAPTCHA_BODY =
    """{"captcha_key":["invalid-input-response"],"captcha_sitekey":"site-1","captcha_service":"hcaptcha","captcha_session_id":"sess-1","captcha_rqtoken":"rq-1"}"""
private val JSON_HEADERS = headersOf(HttpHeaders.ContentType, "application/json")
private const val URL = "https://discord.com/api/v9/auth/login"

private fun clientFor(captcha: CaptchaCoordinator, engine: MockEngine): HttpClient =
    HttpClient(engine) {
        install(LampcordCaptchaPlugin) {
            coordinator = captcha
        }
    }

class CaptchaRetryTest {
    @Test
    fun retriesWithCaptchaHeadersAfterSolve() = runBlocking {
        val coordinator = CaptchaCoordinator()
        val seenKeys = mutableListOf<String?>()
        val seenSessions = mutableListOf<String?>()
        val engine = MockEngine { request ->
            seenKeys += request.headers["X-Captcha-Key"]
            seenSessions += request.headers["X-Captcha-Session-Id"]
            if (request.headers["X-Captcha-Key"] == "token-1") {
                respond("ok", HttpStatusCode.OK)
            } else {
                respond(CAPTCHA_BODY, HttpStatusCode.BadRequest, JSON_HEADERS)
            }
        }
        val client = clientFor(coordinator, engine)
        val solver = launch {
            coordinator.pending.filterNotNull().first()
            coordinator.complete("token-1")
        }
        val response = withTimeout(5_000) {
            client.post(URL) { setBody("{}") }
        }
        solver.join()
        assertEquals(HttpStatusCode.OK, response.status)
        assertEquals(listOf(null, "token-1"), seenKeys)
        assertEquals(listOf(null, "sess-1"), seenSessions)
    }

    @Test
    fun cancelledChallengeThrows() = runBlocking {
        val coordinator = CaptchaCoordinator()
        val engine = MockEngine { respond(CAPTCHA_BODY, HttpStatusCode.BadRequest, JSON_HEADERS) }
        val client = clientFor(coordinator, engine)
        launch {
            coordinator.pending.filterNotNull().first()
            coordinator.cancel()
        }
        assertFailsWith<CaptchaCancelledException> {
            withTimeout(5_000) { client.post(URL) { setBody("{}") } }
        }
        Unit
    }

    @Test
    fun givesUpAfterMaxAttempts() = runBlocking {
        val coordinator = CaptchaCoordinator()
        val engine = MockEngine { respond(CAPTCHA_BODY, HttpStatusCode.BadRequest, JSON_HEADERS) }
        val client = clientFor(coordinator, engine)
        var solves = 0
        val solver = launch {
            repeat(3) {
                coordinator.pending.filterNotNull().first { req -> !req.result.isCompleted }
                solves++
                coordinator.complete("token-$it")
            }
        }
        val response = withTimeout(5_000) { client.post(URL) { setBody("{}") } }
        solver.join()
        assertEquals(HttpStatusCode.BadRequest, response.status)
        assertEquals(3, solves)
    }
}
