package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

@Serializable
data class LoginRequest(
    val login: String,
    val password: String,
    val undelete: Boolean = false,
    val login_source: String? = null,
    val gift_code_sku_id: String? = null,
    val captcha_key: String? = null,
    val captcha_rqtoken: String? = null
)

@Serializable
data class LoginResponse(
    val user_id: String? = null,
    val token: String? = null,
    val ticket: String? = null,
    val mfa: Boolean? = null,
    val user_settings: ModelUserSettingsBootstrap? = null,
    val required_actions: List<JsonElement>? = null,
    val code: Int? = null, // Error code (60003 for MFA required)
    val message: String? = null,
    val totp: Boolean? = null,
    val sms: Boolean? = null,
    val backup: Boolean? = null
)

@Serializable
data class ModelUserSettingsBootstrap(
    val theme: String? = null,
    val locale: String? = null
)

@Serializable
data class MFALoginRequest(
    val code: String,
    val ticket: String,
    val login_source: String? = null,
    val gift_code_sku_id: String? = null
)

@Serializable
data class FingerprintResponse(
    val fingerprint: String? = null
)
