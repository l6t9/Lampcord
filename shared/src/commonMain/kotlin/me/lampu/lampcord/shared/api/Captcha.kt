package me.lampu.lampcord.shared.api

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

const val RECAPTCHA_ENTERPRISE_ACTION = "login"

enum class CaptchaService(val wireName: String, val fixedSiteKey: String?) {
    HCAPTCHA("hcaptcha", null),
    RECAPTCHA("recaptcha", "6Lef5iQTAAAAAKeIvIY-DeexoO3gj7ryl9rLMEnn"),
    RECAPTCHA_ENTERPRISE("recaptcha_enterprise", "6LeYqFcqAAAAAD6iZesmNgVulsO4PkpBdr6NVG6M");

    companion object {
        fun fromWireName(name: String?): CaptchaService? =
            if (name == null) HCAPTCHA else entries.firstOrNull { it.wireName == name }
    }
}

data class CaptchaChallenge(
    val sitekey: String,
    val service: CaptchaService,
    val rqdata: String?,
    val rqtoken: String?,
    val sessionId: String?,
    val shouldServeInvisible: Boolean,
) {
    val isInvisible: Boolean
        get() = shouldServeInvisible || service == CaptchaService.RECAPTCHA_ENTERPRISE
}

fun isCaptchaResponseBody(body: String): Boolean {
    val obj = runCatching { Json.parseToJsonElement(body).jsonObject }.getOrNull() ?: return false
    return obj.containsKey("captcha_key")
}

fun parseCaptchaChallenge(body: String): CaptchaChallenge? {
    val obj = runCatching { Json.parseToJsonElement(body).jsonObject }.getOrNull() ?: return null
    if (!obj.containsKey("captcha_key")) return null
    val service = CaptchaService.fromWireName(obj["captcha_service"]?.jsonPrimitive?.contentOrNull) ?: return null
    val sitekey = obj["captcha_sitekey"]?.jsonPrimitive?.contentOrNull ?: service.fixedSiteKey ?: return null
    return CaptchaChallenge(
        sitekey = sitekey,
        service = service,
        rqdata = obj["captcha_rqdata"]?.jsonPrimitive?.contentOrNull,
        rqtoken = obj["captcha_rqtoken"]?.jsonPrimitive?.contentOrNull,
        sessionId = obj["captcha_session_id"]?.jsonPrimitive?.contentOrNull,
        shouldServeInvisible = obj["should_serve_invisible"]?.jsonPrimitive?.booleanOrNull ?: false,
    )
}
