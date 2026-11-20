package me.lampu.lampcord.shared.api

import me.lampu.lampcord.shared.ui.components.captcha.captchaPageHtml
import me.lampu.lampcord.shared.ui.components.captcha.parseCaptchaBridgeEvent
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
        val html = captchaPageHtml(challenge, "#000000", "dark")
        assertTrue(html.contains("recaptcha/enterprise.js?render=6LeYqFcqAAAAAD6iZesmNgVulsO4PkpBdr6NVG6M"))
        assertTrue(html.contains("grecaptcha.enterprise.execute(SITEKEY"))
    }

    @Test
    fun invisibleHcaptchaRendersInvisibleAndExecutes() {
        val challenge = assertNotNullChallenge(
            """{"captcha_key":["x"],"captcha_sitekey":"k","captcha_service":"hcaptcha","should_serve_invisible":true}""",
        )
        val html = captchaPageHtml(challenge, "#000000", "dark")
        assertTrue(html.contains("opts.size = 'invisible'"))
        assertTrue(html.contains("hcaptcha.execute(widgetId"))
        assertTrue(html.contains("hcaptcha.render('captcha', opts)"))
    }

    @Test
    fun rqdataIsAppliedWithSetDataInsteadOfExecuteOptions() {
        val challenge = assertNotNullChallenge(
            """{"captcha_key":["x"],"captcha_sitekey":"k","captcha_service":"hcaptcha","captcha_rqdata":"rq-data","should_serve_invisible":true}""",
        )
        val html = captchaPageHtml(challenge, "#000000", "dark")
        assertTrue(html.contains("hcaptcha.setData(widgetId, { rqdata: RQDATA })"))
        assertTrue(html.contains("hcaptcha.execute(widgetId);"))
        assertFalse(html.contains("hcaptcha.execute(widgetId, { rqdata: RQDATA })"))
        assertTrue(html.contains("if (!INVISIBLE || widgetId === null"))
    }

    @Test
    fun pageHasRealContainerAndNoTitleMessaging() {
        val challenge = assertNotNullChallenge(
            """{"captcha_key":["x"],"captcha_sitekey":"k","captcha_service":"hcaptcha"}""",
        )
        val html = captchaPageHtml(challenge, "#000000", "dark")
        assertTrue(html.contains("""<div id="captcha"></div>"""))
        assertTrue(html.contains("min-height: 78px"))
        assertTrue(html.contains("callNative('lampcord.captcha'"))
        assertFalse(html.contains("doc.title"))
        assertFalse(html.contains("overlay"))
    }

    @Test
    fun bridgeEventParsesKindSeqAndValue() {
        val event = parseCaptchaBridgeEvent("""{"seq":4,"kind":"token","value":"abc"}""")
        assertEquals(4L, event?.seq)
        assertEquals("token", event?.kind)
        assertEquals("abc", event?.value)
    }

    @Test
    fun bridgeEventAllowsNullValue() {
        val event = parseCaptchaBridgeEvent("""{"seq":1,"kind":"ready","value":null}""")
        assertEquals("ready", event?.kind)
        assertNull(event?.value)
    }

    @Test
    fun malformedBridgeEventIsIgnored() {
        assertNull(parseCaptchaBridgeEvent("not json"))
        assertNull(parseCaptchaBridgeEvent("""{"kind":"ready"}"""))
    }

    private fun assertNotNullChallenge(body: String): CaptchaChallenge =
        parseCaptchaChallenge(body) ?: error("Expected a captcha challenge for: $body")
}
