package me.lampu.lampcord.shared.api

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

data class CaptchaChallenge(
    val sitekey: String,
    val service: String,
    val rqdata: String?,
    val rqtoken: String?,
    val sessionId: String? = null,
)

fun parseCaptchaChallenge(body: String): CaptchaChallenge? {
    val obj = runCatching { Json.parseToJsonElement(body).jsonObject }.getOrNull() ?: return null
    if (!obj.containsKey("captcha_key")) return null
    val sitekey = obj["captcha_sitekey"]?.jsonPrimitive?.contentOrNull ?: return null
    return CaptchaChallenge(
        sitekey = sitekey,
        service = obj["captcha_service"]?.jsonPrimitive?.contentOrNull ?: "hcaptcha",
        rqdata = obj["captcha_rqdata"]?.jsonPrimitive?.contentOrNull,
        rqtoken = obj["captcha_rqtoken"]?.jsonPrimitive?.contentOrNull,
        sessionId = obj["captcha_session_id"]?.jsonPrimitive?.contentOrNull,
    )
}
