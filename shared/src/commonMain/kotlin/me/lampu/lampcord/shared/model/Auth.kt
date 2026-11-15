package me.lampu.lampcord.shared.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient
import kotlinx.serialization.json.JsonElement

@Serializable
data class LoginRequest(
    val login: String,
    val password: String,
    val undelete: Boolean = false,
    val login_source: String? = null,
    val gift_code_sku_id: String? = null,
)

@Serializable
data class LoginResponse(
    val user_id: String? = null,
    val token: String? = null,
    val ticket: String? = null,
    val login_instance_id: String? = null,
    val mfa: Boolean? = null,
    val user_settings: ModelUserSettingsBootstrap? = null,
    val required_actions: List<JsonElement>? = null,
    val code: Int? = null,
    val message: String? = null,
    val totp: Boolean? = null,
    val sms: Boolean? = null,
    val backup: Boolean? = null,
    val webauthn: String? = null,
    val suspended_user_token: String? = null,
    @Transient val failure: LoginFailure? = null,
    @Transient val captchaCancelled: Boolean = false
)

enum class LoginFailure {
    INVALID_CREDENTIALS,
    RATE_LIMITED,
    CAPTCHA_REJECTED,
    VERIFICATION_REQUIRED,
    ACCOUNT_DISABLED,
    ACCOUNT_SCHEDULED_FOR_DELETION,
    SUSPENDED,
    PASSWORD_UPDATE_REQUIRED,
    SERVER_ERROR,
    NETWORK,
    UNKNOWN
}

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
    val gift_code_sku_id: String? = null,
    val login_instance_id: String? = null,
)

@Serializable
data class FingerprintResponse(
    val fingerprint: String? = null
)
