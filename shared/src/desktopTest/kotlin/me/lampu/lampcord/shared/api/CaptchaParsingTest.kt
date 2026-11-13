package me.lampu.lampcord.shared.api

import me.lampu.lampcord.shared.ui.components.captcha.captchaInjectionScript
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class CaptchaParsingTest {
    @Test
    fun parsesHcaptchaChallenge() {
        val body = """{"captcha_key":["invalid-input-response"],"captcha_sitekey":"f5561ba9-8f1e-40ca-9b5b-a0b3f719ef34","captcha_service":"hcaptcha"}"""
        val challenge = assertNotNullChallenge(body)
        assertEquals(CaptchaService.HCAPTCHA, challenge.service)
        assertEquals("f5561ba9-8f1e-40ca-9b5b-a0b3f719ef34", challenge.sitekey)
        assertFalse(challenge.isInvisible)
    }

    @Test
    fun invisibleFlagIsRespected() {
        val body = """{"captcha_key":["x"],"captcha_sitekey":"k","captcha_service":"hcaptcha","should_serve_invisible":true}"""
        assertTrue(assertNotNullChallenge(body).isInvisible)
    }

    @Test
    fun enterpriseUsesFixedSiteKeyWhenMissing() {
        val body = """{"captcha_key":["x"],"captcha_service":"recaptcha_enterprise"}"""
        val challenge = assertNotNullChallenge(body)
        assertEquals("6LeYqFcqAAAAAD6iZesmNgVulsO4PkpBdr6NVG6M", challenge.sitekey)
        assertTrue(challenge.isInvisible)
    }

    @Test
    fun unknownServiceIsNotAChallenge() {
        val body = """{"captcha_key":["x"],"captcha_sitekey":"k","captcha_service":"mystery"}"""
        assertNull(parseCaptchaChallenge(body))
    }

    @Test
    fun bodyWithoutCaptchaKeyIsNotAChallenge() {
        val body = """{"message":"Invalid Form Body","code":50035}"""
        assertNull(parseCaptchaChallenge(body))
        assertFalse(isCaptchaResponseBody(body))
    }

    @Test
    fun enterpriseScriptLoadsEnterpriseLibraryAndExecutesWithAction() {
        val challenge = assertNotNullChallenge(
            """{"captcha_key":["x"],"captcha_service":"recaptcha_enterprise"}""",
        )
        val script = captchaInjectionScript(challenge, "#000000", "dark")
        assertTrue(script.contains("recaptcha/enterprise.js?render=6LeYqFcqAAAAAD6iZesmNgVulsO4PkpBdr6NVG6M"))
        assertTrue(script.contains("grecaptcha.enterprise.execute(SITEKEY"))
    }

    @Test
    fun invisibleHcaptchaRendersInvisibleAndExecutes() {
        val challenge = assertNotNullChallenge(
            """{"captcha_key":["x"],"captcha_sitekey":"k","captcha_service":"hcaptcha","should_serve_invisible":true}""",
        )
        val script = captchaInjectionScript(challenge, "#000000", "dark")
        assertTrue(script.contains("opts.size = 'invisible'"))
        assertTrue(script.contains("hcaptcha.execute(widgetId"))
    }

    private fun assertNotNullChallenge(body: String): CaptchaChallenge =
        parseCaptchaChallenge(body) ?: error("Expected a captcha challenge for: $body")
}
