package me.lampu.lampcord.shared.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AlertDialogDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.api.ForgotPasswordResult
import me.lampu.lampcord.shared.api.RemoteAuthClient
import me.lampu.lampcord.shared.api.RemoteAuthState
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.back_to_login
import me.lampu.lampcord.shared.generated.resources.enter_phone_or_email
import me.lampu.lampcord.shared.generated.resources.error
import me.lampu.lampcord.shared.generated.resources.form_label_password
import me.lampu.lampcord.shared.generated.resources.language
import me.lampu.lampcord.shared.generated.resources.language_select
import me.lampu.lampcord.shared.generated.resources.lampcord_language_system
import me.lampu.lampcord.shared.generated.resources.lampcord_language_system_description
import me.lampu.lampcord.shared.generated.resources.lampcord_login_backup_code_label
import me.lampu.lampcord.shared.generated.resources.lampcord_login_backup_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_backup_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_close
import me.lampu.lampcord.shared.generated.resources.lampcord_login_error_captcha_cancelled
import me.lampu.lampcord.shared.generated.resources.lampcord_login_error_enter_token
import me.lampu.lampcord.shared.generated.resources.lampcord_login_error_failed
import me.lampu.lampcord.shared.generated.resources.lampcord_login_error_invalid
import me.lampu.lampcord.shared.generated.resources.lampcord_login_error_invalid_mfa
import me.lampu.lampcord.shared.generated.resources.lampcord_login_error_unexpected
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_empty
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_password
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_phone_unsupported
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_send
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_sent_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_sent_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_forgot_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_mfa_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_mfa_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_other_login_qr
import me.lampu.lampcord.shared.generated.resources.lampcord_login_other_methods
import me.lampu.lampcord.shared.generated.resources.lampcord_login_other_options
import me.lampu.lampcord.shared.generated.resources.lampcord_login_passkey_only
import me.lampu.lampcord.shared.generated.resources.lampcord_login_password_hide
import me.lampu.lampcord.shared.generated.resources.lampcord_login_password_show
import me.lampu.lampcord.shared.generated.resources.lampcord_login_qr_approve
import me.lampu.lampcord.shared.generated.resources.lampcord_login_qr_prompt
import me.lampu.lampcord.shared.generated.resources.lampcord_login_resend_sms
import me.lampu.lampcord.shared.generated.resources.lampcord_login_restore_account
import me.lampu.lampcord.shared.generated.resources.lampcord_login_sms_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_sms_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_token_hide
import me.lampu.lampcord.shared.generated.resources.lampcord_login_token_label
import me.lampu.lampcord.shared.generated.resources.lampcord_login_token_show
import me.lampu.lampcord.shared.generated.resources.lampcord_login_use_authenticator
import me.lampu.lampcord.shared.generated.resources.lampcord_login_use_backup
import me.lampu.lampcord.shared.generated.resources.lampcord_login_use_sms
import me.lampu.lampcord.shared.generated.resources.lampcord_login_verify_email_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_verify_empty
import me.lampu.lampcord.shared.generated.resources.lampcord_login_verify_link_label
import me.lampu.lampcord.shared.generated.resources.lampcord_login_verify_phone_subtitle
import me.lampu.lampcord.shared.generated.resources.lampcord_login_verify_title
import me.lampu.lampcord.shared.generated.resources.lampcord_login_with_token
import me.lampu.lampcord.shared.generated.resources.login
import me.lampu.lampcord.shared.generated.resources.retry
import me.lampu.lampcord.shared.generated.resources.user_settings_avatar
import me.lampu.lampcord.shared.generated.resources.verify
import me.lampu.lampcord.shared.i18n.LANGUAGE_SYSTEM
import me.lampu.lampcord.shared.i18n.SUPPORTED_LANGUAGES
import me.lampu.lampcord.shared.model.LoginFailure
import me.lampu.lampcord.shared.model.LoginResponse
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.OtpInput
import me.lampu.lampcord.shared.ui.components.PlatformBackHandler
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.handCursor
import me.lampu.lampcord.shared.utils.getPlatformName
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject
import qrcode.QRCode
import qrcode.raw.ErrorCorrectionLevel
import kotlin.math.min

private enum class MfaMethod(val apiType: String) {
    Totp("totp"),
    Sms("sms"),
    Backup("backup")
}

private enum class VerificationMode { Phone, EmailLink }

private enum class LoginStep { Credentials, Token, Qr, Mfa, Verification, Forgot, ForgotSent }

private const val SMS_LOGIN_ENABLED = false

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun LoginScreen(
    modifier: Modifier = Modifier,
    sessionManager: SessionManager = koinInject(),
    remoteAuthClient: RemoteAuthClient = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    tokenStore: me.lampu.lampcord.shared.state.TokenStore = koinInject(),
    onBack: (() -> Unit)? = null,
    connectOnSuccess: Boolean = true,
    onLoginSuccess: () -> Unit
) {
    val isMobile = remember {
        getPlatformName().let { it == "android" || it == "ios" }
    }

    var step by remember { mutableStateOf(LoginStep.Credentials) }
    var showOtherOptions by remember { mutableStateOf(false) }
    var showLanguagePicker by remember { mutableStateOf(false) }

    var login by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var passwordVisible by remember { mutableStateOf(false) }
    var token by remember { mutableStateOf("") }
    var tokenVisible by remember { mutableStateOf(false) }
    var isLoading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var failure by remember { mutableStateOf<LoginFailure?>(null) }

    var mfaTicket by remember { mutableStateOf<String?>(null) }
    var mfaInstanceId by remember { mutableStateOf<String?>(null) }
    var mfaCode by remember { mutableStateOf("") }
    var mfaMethods by remember { mutableStateOf<List<MfaMethod>>(emptyList()) }
    var mfaMethod by remember { mutableStateOf(MfaMethod.Totp) }
    var mfaPasskeyOnly by remember { mutableStateOf(false) }
    var mfaPhone by remember { mutableStateOf<String?>(null) }

    var verificationMode by remember { mutableStateOf<VerificationMode?>(null) }
    var verificationTarget by remember { mutableStateOf("") }
    var verificationInput by remember { mutableStateOf("") }
    var forgotLogin by remember { mutableStateOf("") }

    val passwordFocus = remember { FocusRequester() }
    val scope = rememberCoroutineScope()
    val remoteAuthState by remoteAuthClient.state.collectAsState()
    val staleRemoteAuth = remember { remoteAuthClient.state.value as? RemoteAuthState.Finished }

    val enterTokenError = stringResource(Res.string.lampcord_login_error_enter_token)
    val loginFailedError = stringResource(Res.string.lampcord_login_error_failed)
    val invalidLoginError = stringResource(Res.string.lampcord_login_error_invalid)
    val invalidMfaError = stringResource(Res.string.lampcord_login_error_invalid_mfa)
    val captchaCancelledError = stringResource(Res.string.lampcord_login_error_captcha_cancelled)
    val unexpectedError = stringResource(Res.string.lampcord_login_error_unexpected)
    val verificationEmptyError = stringResource(Res.string.lampcord_login_verify_empty)
    val forgotEmptyError = stringResource(Res.string.lampcord_login_forgot_empty)
    val phoneUnsupportedError = stringResource(Res.string.lampcord_login_forgot_phone_unsupported)

    suspend fun completeAuth(tokenValue: String): Boolean {
        return if (connectOnSuccess) {
            sessionManager.connect(tokenValue)
            onLoginSuccess()
            true
        } else {
            val user = sessionManager.authApi.getCurrentUser(tokenValue)
            if (user != null) {
                tokenStore.addAccount(tokenValue, user)
                onLoginSuccess()
                true
            } else {
                errorMessage = loginFailedError
                false
            }
        }
    }

    val passwordError = step == LoginStep.Credentials && failure == LoginFailure.INVALID_CREDENTIALS
    val canRestoreAccount = step == LoginStep.Credentials &&
        (failure == LoginFailure.ACCOUNT_DISABLED || failure == LoginFailure.ACCOUNT_SCHEDULED_FOR_DELETION)
    val bannerError = if (passwordError) null else errorMessage


    fun clearError() {
        errorMessage = null
        failure = null
    }

    fun openStep(next: LoginStep) {
        clearError()
        showOtherOptions = false
        step = next
    }

    fun busy(block: suspend () -> Unit) {
        if (isLoading) return
        isLoading = true
        scope.launch {
            try {
                block()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                errorMessage = e.message ?: unexpectedError
            } finally {
                isLoading = false
            }
        }
    }

    fun resetMfa() {
        mfaTicket = null
        mfaInstanceId = null
        mfaMethods = emptyList()
        mfaPasskeyOnly = false
        mfaPhone = null
        mfaCode = ""
    }

    fun leaveMfa() {
        resetMfa()
        openStep(LoginStep.Credentials)
    }

    fun leaveVerification() {
        verificationMode = null
        verificationInput = ""
        openStep(LoginStep.Credentials)
    }

    suspend fun sendSmsCode() {
        val ticket = mfaTicket ?: return
        sessionManager.sendMfaSms(ticket)
            .onSuccess { mfaPhone = it }
            .onFailure { errorMessage = it.message ?: unexpectedError }
    }

    suspend fun switchMfaMethod(method: MfaMethod) {
        mfaMethod = method
        mfaCode = ""
        clearError()
        if (method == MfaMethod.Sms) sendSmsCode()
    }

    suspend fun startMfa(response: LoginResponse) {
        val available = buildList {
            if (SMS_LOGIN_ENABLED && response.sms == true) add(MfaMethod.Sms)
            if (response.totp == true) add(MfaMethod.Totp)
            if (response.backup == true) add(MfaMethod.Backup)
        }
        val methods = when {
            available.isNotEmpty() -> available
            response.webauthn != null -> emptyList()
            else -> listOf(MfaMethod.Totp)
        }
        mfaTicket = response.ticket
        mfaInstanceId = response.login_instance_id
        mfaMethods = methods
        mfaPasskeyOnly = methods.isEmpty()
        mfaPhone = null
        mfaCode = ""
        clearError()
        step = LoginStep.Mfa
        methods.firstOrNull()?.let { switchMfaMethod(it) }
    }

    suspend fun verifyMfa() {
        val ticket = mfaTicket ?: return
        val res = sessionManager.verifyMFA(
            code = mfaCode.trim(),
            ticket = ticket,
            type = mfaMethod.apiType,
            loginInstanceId = mfaInstanceId
        )
        if (res?.token != null) {
            password = ""
            completeAuth(res.token)
        } else {
            errorMessage = res?.message ?: invalidMfaError
            mfaCode = ""
        }
    }

    suspend fun credentialLogin(undelete: Boolean) {
        val response = sessionManager.login(login.trim(), password, undelete)
        when {
            response == null -> errorMessage = loginFailedError
            response.captchaCancelled -> errorMessage = captchaCancelledError
            response.token != null -> {
                password = ""
                completeAuth(response.token)
            }
            response.mfa == true && response.ticket != null -> startMfa(response)
            response.failure == LoginFailure.VERIFICATION_REQUIRED -> {
                verificationTarget = login.trim()
                verificationMode = if (login.contains('@')) VerificationMode.EmailLink else VerificationMode.Phone
                verificationInput = ""
                clearError()
                step = LoginStep.Verification
            }
            else -> {
                errorMessage = response.message ?: invalidLoginError
                failure = response.failure
            }
        }
    }

    suspend fun completeVerification() {
        val mode = verificationMode ?: return
        val input = verificationInput.trim()
        if (input.isEmpty()) {
            errorMessage = verificationEmptyError
            return
        }
        val verifiedToken = if (mode == VerificationMode.Phone) {
            val verified = sessionManager.verifyLoginPhone(verificationTarget, input)
            if (verified.isFailure) {
                errorMessage = verified.exceptionOrNull()?.message ?: unexpectedError
                verificationInput = ""
                return
            }
            verified.getOrThrow()
        } else {
            verificationTokenFrom(input)
        }
        val authorized = sessionManager.authorizeLoginIp(verifiedToken)
        if (authorized.isFailure) {
            errorMessage = authorized.exceptionOrNull()?.message ?: unexpectedError
            return
        }
        verificationMode = null
        verificationInput = ""
        openStep(LoginStep.Credentials)
        credentialLogin(undelete = false)
    }

    suspend fun submitForgot() {
        val target = forgotLogin.trim()
        if (target.isEmpty()) {
            errorMessage = forgotEmptyError
            return
        }
        when (val result = sessionManager.forgotPassword(target)) {
            ForgotPasswordResult.Sent -> openStep(LoginStep.ForgotSent)
            ForgotPasswordResult.PhoneVerificationRequired -> errorMessage = phoneUnsupportedError
            is ForgotPasswordResult.Failed -> errorMessage = result.message
        }
    }

    suspend fun resendVerificationCode() {
        sessionManager.resendLoginPhoneCode(verificationTarget)
            .onFailure { errorMessage = it.message ?: unexpectedError }
    }

    fun submit(undelete: Boolean = false) = busy {
        when (step) {
            LoginStep.Verification -> completeVerification()
            LoginStep.Mfa -> verifyMfa()
            LoginStep.Credentials -> credentialLogin(undelete)
            LoginStep.Token -> {
                val authToken = token.trim()
                if (authToken.isBlank()) {
                    errorMessage = enterTokenError
                } else {
                    if (completeAuth(authToken)) {
                        token = ""
                    }
                }
            }
            LoginStep.Qr -> Unit
            LoginStep.Forgot -> submitForgot()
            LoginStep.ForgotSent -> Unit
        }
    }

    LaunchedEffect(mfaCode) {
        if (step == LoginStep.Mfa && mfaMethod != MfaMethod.Backup && mfaCode.length == 6 && !isLoading) {
            submit()
        }
    }

    LaunchedEffect(verificationInput) {
        if (step == LoginStep.Verification && verificationMode == VerificationMode.Phone &&
            verificationInput.length == 6 && !isLoading
        ) {
            submit()
        }
    }

    LaunchedEffect(step) {
        if (step == LoginStep.Qr || (!isMobile && step == LoginStep.Credentials)) {
            remoteAuthClient.start()
        } else {
            remoteAuthClient.stop()
        }
    }

    DisposableEffect(Unit) {
        onDispose { remoteAuthClient.stop() }
    }

    LaunchedEffect(remoteAuthState) {
        val state = remoteAuthState
        if (state is RemoteAuthState.Finished && state != staleRemoteAuth) {
            completeAuth(state.token)
        }
    }

    val stepBack: (() -> Unit)? = when (step) {
        LoginStep.Credentials -> onBack
        LoginStep.Mfa -> ({ leaveMfa() })
        LoginStep.Verification -> ({ leaveVerification() })
        else -> ({ openStep(LoginStep.Credentials) })
    }
    val backAction: (() -> Unit)? = stepBack?.let { action ->
        val guarded: () -> Unit = { if (!isLoading) action() }
        guarded
    }

    PlatformBackHandler(enabled = backAction != null) { backAction?.invoke() }

    Surface(
        modifier = modifier
            .fillMaxSize()
            .focusable()
            .onPreviewKeyEvent { event ->
                val back = backAction
                if (back != null && event.type == KeyEventType.KeyDown && event.key == Key.Escape) {
                    back()
                    true
                } else {
                    false
                }
            },
        color = MaterialTheme.colorScheme.background
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            IconButton(
                onClick = { showLanguagePicker = true },
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .statusBarsPadding()
                    .padding(12.dp)
                    .handCursor()
            ) {
                Icon(
                    imageVector = Icons.Rounded.Language,
                    contentDescription = stringResource(Res.string.language)
                )
            }

            backAction?.let { onBack ->
                IconButton(
                    onClick = onBack,
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .statusBarsPadding()
                        .padding(12.dp)
                        .handCursor()
                ) {
                    Icon(
                        imageVector = Icons.Rounded.ArrowBack,
                        contentDescription = stringResource(Res.string.back_to_login)
                    )
                }
            }

            Column(
                modifier = Modifier
                    .align(Alignment.Center)
                    .widthIn(max = 480.dp)
                    .fillMaxWidth()
                    .imePadding()
                    .verticalScroll(rememberScrollState())
                    .padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Text(
                    text = "          へ   ♡     ╱|、\n" +
                           "     ૮  -   ՛ )      (`   -  7\n" +
                           "       /   ⁻  ៸|       |、⁻〵\n" +
                           " 乀 (ˍ, ل ل        じしˍ,)ノ",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.primary,
                    lineHeight = 22.sp
                )

                Spacer(modifier = Modifier.height(16.dp))

                AnimatedContent(
                    targetState = step,
                    modifier = Modifier.fillMaxWidth().animateContentSize(),
                    transitionSpec = {
                        (fadeIn(animationSpec = tween(220)) +
                            slideInHorizontally(animationSpec = tween(220)) { it / 8 }) togetherWith
                            (fadeOut(animationSpec = tween(160)) +
                                slideOutHorizontally(animationSpec = tween(160)) { -it / 8 })
                    },
                    label = "login-step"
                ) { current ->
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                            when (current) {
                                LoginStep.Credentials -> {
                                    OutlinedTextField(
                                        value = login,
                                        onValueChange = { login = it; clearError() },
                                        label = { Text(stringResource(Res.string.enter_phone_or_email)) },
                                        modifier = Modifier.fillMaxWidth(),
                                        singleLine = true,
                                        enabled = !isLoading,
                                        keyboardOptions = KeyboardOptions(
                                            capitalization = KeyboardCapitalization.None,
                                            keyboardType = KeyboardType.Text,
                                            imeAction = ImeAction.Next
                                        ),
                                        keyboardActions = KeyboardActions(onNext = { passwordFocus.requestFocus() })
                                    )
                                    Spacer(modifier = Modifier.height(12.dp))
                                    OutlinedTextField(
                                        value = password,
                                        onValueChange = { password = it; clearError() },
                                        label = { Text(stringResource(Res.string.form_label_password)) },
                                        visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
                                        modifier = Modifier.fillMaxWidth().focusRequester(passwordFocus),
                                        singleLine = true,
                                        enabled = !isLoading,
                                        isError = passwordError,
                                        supportingText = if (passwordError) ({ Text(errorMessage.orEmpty()) }) else null,
                                        keyboardOptions = KeyboardOptions(
                                            keyboardType = KeyboardType.Password,
                                            imeAction = ImeAction.Done
                                        ),
                                        keyboardActions = KeyboardActions(onDone = { submit() }),
                                        trailingIcon = {
                                            IconButton(onClick = { passwordVisible = !passwordVisible }) {
                                                Icon(
                                                    imageVector = if (passwordVisible) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                                                    contentDescription = if (passwordVisible) {
                                                        stringResource(Res.string.lampcord_login_password_hide)
                                                    } else {
                                                        stringResource(Res.string.lampcord_login_password_show)
                                                    }
                                                )
                                            }
                                        }
                                    )
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.End
                                    ) {
                                        TextButton(
                                            onClick = {
                                                forgotLogin = login.trim()
                                                openStep(LoginStep.Forgot)
                                            },
                                            modifier = Modifier.handCursor()
                                        ) {
                                            Text(stringResource(Res.string.lampcord_login_forgot_password))
                                        }
                                    }
                                    ErrorBanner(bannerError)
                                    PrimaryButton(
                                        text = stringResource(Res.string.login),
                                        enabled = !isLoading,
                                        loading = isLoading,
                                        onClick = { submit() }
                                    )
                                    if (canRestoreAccount) {
                                        Spacer(modifier = Modifier.height(8.dp))
                                        FilledTonalButton(
                                            onClick = { submit(undelete = true) },
                                            enabled = !isLoading,
                                            modifier = Modifier.fillMaxWidth().height(48.dp).handCursor(),
                                            shape = MaterialTheme.shapes.large
                                        ) {
                                            Text(stringResource(Res.string.lampcord_login_restore_account))
                                        }
                                    }
                                    if (isMobile) {
                                        OtherOptionsToggle(
                                            label = stringResource(Res.string.lampcord_login_other_options),
                                            onToggle = { showOtherOptions = !showOtherOptions }
                                        )
                                        AnimatedVisibility(
                                            visible = showOtherOptions,
                                            enter = fadeIn() + expandVertically(),
                                            exit = fadeOut() + shrinkVertically()
                                        ) {
                                            LoginOptionButtons(
                                                onToken = { openStep(LoginStep.Token) },
                                                onQr = { openStep(LoginStep.Qr) }
                                            )
                                        }
                                    } else {
                                        LoginOptionButtons(
                                            onToken = { openStep(LoginStep.Token) },
                                            onQr = null
                                        )
                                        Spacer(modifier = Modifier.height(20.dp))
                                        Text(
                                            text = stringResource(Res.string.lampcord_login_qr_prompt),
                                            style = MaterialTheme.typography.bodyMedium,
                                            textAlign = TextAlign.Center,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                        Spacer(modifier = Modifier.height(16.dp))
                                        QrContent(
                                            state = remoteAuthState,
                                            onRetry = { scope.launch { remoteAuthClient.start() } }
                                        )
                                    }
                                }

                                LoginStep.Token -> {
                                    StepHeading(
                                        title = stringResource(Res.string.lampcord_login_with_token),
                                        subtitle = stringResource(Res.string.lampcord_login_subtitle)
                                    )
                                    Spacer(modifier = Modifier.height(20.dp))
                                    OutlinedTextField(
                                        value = token,
                                        onValueChange = { token = it; clearError() },
                                        label = { Text(stringResource(Res.string.lampcord_login_token_label)) },
                                        visualTransformation = if (tokenVisible) VisualTransformation.None else PasswordVisualTransformation(),
                                        modifier = Modifier.fillMaxWidth(),
                                        singleLine = true,
                                        enabled = !isLoading,
                                        keyboardOptions = KeyboardOptions(
                                            capitalization = KeyboardCapitalization.None,
                                            keyboardType = KeyboardType.Password,
                                            imeAction = ImeAction.Done
                                        ),
                                        keyboardActions = KeyboardActions(onDone = { submit() }),
                                        trailingIcon = {
                                            IconButton(onClick = { tokenVisible = !tokenVisible }) {
                                                Icon(
                                                    imageVector = if (tokenVisible) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                                                    contentDescription = if (tokenVisible) {
                                                        stringResource(Res.string.lampcord_login_token_hide)
                                                    } else {
                                                        stringResource(Res.string.lampcord_login_token_show)
                                                    }
                                                )
                                            }
                                        }
                                    )
                                    ErrorBanner(errorMessage)
                                    PrimaryButton(
                                        text = stringResource(Res.string.login),
                                        enabled = !isLoading,
                                        loading = isLoading,
                                        onClick = { submit() }
                                    )
                                }

                                LoginStep.Qr -> {
                                    StepHeading(
                                        title = stringResource(Res.string.lampcord_login_other_login_qr),
                                        subtitle = stringResource(Res.string.lampcord_login_qr_prompt)
                                    )
                                    Spacer(modifier = Modifier.height(20.dp))
                                    QrContent(
                                        state = remoteAuthState,
                                        onRetry = { scope.launch { remoteAuthClient.start() } }
                                    )
                                }

                                LoginStep.Mfa -> {
                                    StepHeading(
                                        title = when (mfaMethod) {
                                            MfaMethod.Totp -> stringResource(Res.string.lampcord_login_mfa_title)
                                            MfaMethod.Sms -> stringResource(Res.string.lampcord_login_sms_title)
                                            MfaMethod.Backup -> stringResource(Res.string.lampcord_login_backup_title)
                                        },
                                        subtitle = when {
                                            mfaPasskeyOnly -> stringResource(Res.string.lampcord_login_passkey_only)
                                            mfaMethod == MfaMethod.Totp -> stringResource(Res.string.lampcord_login_mfa_subtitle)
                                            mfaMethod == MfaMethod.Sms && mfaPhone != null ->
                                                stringResource(Res.string.lampcord_login_sms_subtitle, mfaPhone.orEmpty())
                                            mfaMethod == MfaMethod.Sms -> stringResource(Res.string.lampcord_login_mfa_subtitle)
                                            else -> stringResource(Res.string.lampcord_login_backup_subtitle)
                                        }
                                    )
                                    if (!mfaPasskeyOnly) {
                                        Spacer(modifier = Modifier.height(20.dp))
                                        if (mfaMethod == MfaMethod.Backup) {
                                            OutlinedTextField(
                                                value = mfaCode,
                                                onValueChange = { clearError(); mfaCode = it.take(32) },
                                                label = { Text(stringResource(Res.string.lampcord_login_backup_code_label)) },
                                                modifier = Modifier.fillMaxWidth(),
                                                singleLine = true,
                                                enabled = !isLoading,
                                                isError = errorMessage != null,
                                                keyboardOptions = KeyboardOptions(
                                                    keyboardType = KeyboardType.Text,
                                                    imeAction = ImeAction.Done
                                                ),
                                                keyboardActions = KeyboardActions(onDone = { submit() })
                                            )
                                        } else {
                                            OtpInput(
                                                value = mfaCode,
                                                onValueChange = { clearError(); mfaCode = it },
                                                enabled = !isLoading,
                                                isError = errorMessage != null,
                                                autoFocus = true,
                                                keyboardActions = KeyboardActions(onDone = { submit() })
                                            )
                                        }
                                    }
                                    ErrorBanner(errorMessage)
                                    if (!mfaPasskeyOnly) {
                                        PrimaryButton(
                                            text = stringResource(Res.string.verify),
                                            enabled = !isLoading,
                                            loading = isLoading,
                                            onClick = { submit() }
                                        )
                                        if (mfaMethod == MfaMethod.Sms) {
                                            TextButton(
                                                onClick = { busy { sendSmsCode() } },
                                                enabled = !isLoading,
                                                modifier = Modifier.handCursor()
                                            ) {
                                                Text(stringResource(Res.string.lampcord_login_resend_sms))
                                            }
                                        }
                                    }
                                    if (mfaMethods.size > 1) {
                                        OtherOptionsToggle(
                                            label = stringResource(Res.string.lampcord_login_other_methods),
                                            onToggle = { showOtherOptions = !showOtherOptions }
                                        )
                                        AnimatedVisibility(
                                            visible = showOtherOptions,
                                            enter = fadeIn() + expandVertically(),
                                            exit = fadeOut() + shrinkVertically()
                                        ) {
                                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                                mfaMethods.filter { it != mfaMethod }.forEach { method ->
                                                    TextButton(
                                                        onClick = {
                                                            showOtherOptions = false
                                                            busy { switchMfaMethod(method) }
                                                        },
                                                        enabled = !isLoading,
                                                        modifier = Modifier.handCursor()
                                                    ) {
                                                        Text(
                                                            when (method) {
                                                                MfaMethod.Totp -> stringResource(Res.string.lampcord_login_use_authenticator)
                                                                MfaMethod.Sms -> stringResource(Res.string.lampcord_login_use_sms)
                                                                MfaMethod.Backup -> stringResource(Res.string.lampcord_login_use_backup)
                                                            }
                                                        )
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                LoginStep.Forgot -> {
                                    StepHeading(
                                        title = stringResource(Res.string.lampcord_login_forgot_title),
                                        subtitle = stringResource(Res.string.lampcord_login_forgot_subtitle)
                                    )
                                    Spacer(modifier = Modifier.height(20.dp))
                                    OutlinedTextField(
                                        value = forgotLogin,
                                        onValueChange = { forgotLogin = it; clearError() },
                                        label = { Text(stringResource(Res.string.enter_phone_or_email)) },
                                        modifier = Modifier.fillMaxWidth(),
                                        singleLine = true,
                                        enabled = !isLoading,
                                        keyboardOptions = KeyboardOptions(
                                            capitalization = KeyboardCapitalization.None,
                                            keyboardType = KeyboardType.Text,
                                            imeAction = ImeAction.Done
                                        ),
                                        keyboardActions = KeyboardActions(onDone = { submit() })
                                    )
                                    ErrorBanner(errorMessage)
                                    PrimaryButton(
                                        text = stringResource(Res.string.lampcord_login_forgot_send),
                                        enabled = !isLoading,
                                        loading = isLoading,
                                        onClick = { submit() }
                                    )
                                }

                                LoginStep.ForgotSent -> {
                                    StepHeading(
                                        title = stringResource(Res.string.lampcord_login_forgot_sent_title),
                                        subtitle = stringResource(Res.string.lampcord_login_forgot_sent_subtitle)
                                    )
                                }

                                LoginStep.Verification -> {
                                    val mode = verificationMode
                                    StepHeading(
                                        title = stringResource(Res.string.lampcord_login_verify_title),
                                        subtitle = if (mode == VerificationMode.Phone) {
                                            stringResource(Res.string.lampcord_login_verify_phone_subtitle, verificationTarget)
                                        } else {
                                            stringResource(Res.string.lampcord_login_verify_email_subtitle)
                                        }
                                    )
                                    Spacer(modifier = Modifier.height(20.dp))
                                    if (mode == VerificationMode.Phone) {
                                        OtpInput(
                                            value = verificationInput,
                                            onValueChange = { clearError(); verificationInput = it },
                                            enabled = !isLoading,
                                            isError = errorMessage != null,
                                            autoFocus = true,
                                            keyboardActions = KeyboardActions(onDone = { submit() })
                                        )
                                    } else {
                                        OutlinedTextField(
                                            value = verificationInput,
                                            onValueChange = { clearError(); verificationInput = it.trim() },
                                            label = { Text(stringResource(Res.string.lampcord_login_verify_link_label)) },
                                            modifier = Modifier.fillMaxWidth(),
                                            singleLine = true,
                                            enabled = !isLoading,
                                            keyboardOptions = KeyboardOptions(
                                                keyboardType = KeyboardType.Text,
                                                imeAction = ImeAction.Done
                                            ),
                                            keyboardActions = KeyboardActions(onDone = { submit() })
                                        )
                                    }
                                    ErrorBanner(errorMessage)
                                    PrimaryButton(
                                        text = stringResource(Res.string.verify),
                                        enabled = !isLoading,
                                        loading = isLoading,
                                        onClick = { submit() }
                                    )
                                    if (mode == VerificationMode.Phone) {
                                        TextButton(
                                            onClick = { busy { resendVerificationCode() } },
                                            enabled = !isLoading,
                                            modifier = Modifier.handCursor()
                                        ) {
                                            Text(stringResource(Res.string.lampcord_login_resend_sms))
                                        }
                                    }
                                }
                            }
                        }
                    }
            }
        }
    }

    if (showLanguagePicker) {
        AlertDialog(
            onDismissRequest = { showLanguagePicker = false },
            title = { Text(stringResource(Res.string.language_select)) },
            text = {
                Column(
                    modifier = Modifier.heightIn(max = 420.dp).verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    LanguageOption(
                        title = stringResource(Res.string.lampcord_language_system),
                        description = stringResource(Res.string.lampcord_language_system_description),
                        selected = settingsStore.language == LANGUAGE_SYSTEM,
                        onSelect = {
                            settingsStore.language = LANGUAGE_SYSTEM
                            showLanguagePicker = false
                        }
                    )
                    SUPPORTED_LANGUAGES.forEach { language ->
                        LanguageOption(
                            title = language.nativeName,
                            description = language.englishName,
                            selected = settingsStore.language == language.languageTag,
                            onSelect = {
                                settingsStore.language = language.languageTag
                                showLanguagePicker = false
                            }
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { showLanguagePicker = false }) {
                    Text(stringResource(Res.string.lampcord_login_close))
                }
            }
        )
    }
}

@Composable
private fun StepHeading(title: String, subtitle: String?) {
    Text(
        text = title,
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.Bold,
        textAlign = TextAlign.Center,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.fillMaxWidth()
    )
    if (subtitle != null) {
        Spacer(modifier = Modifier.height(6.dp))
        Text(
            text = subtitle,
            style = MaterialTheme.typography.bodyMedium,
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
private fun LoginOptionButtons(onToken: () -> Unit, onQr: (() -> Unit)?) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        TextButton(onClick = onToken, modifier = Modifier.handCursor()) {
            Text(stringResource(Res.string.lampcord_login_with_token))
        }
        if (onQr != null) {
            TextButton(onClick = onQr, modifier = Modifier.handCursor()) {
                Text(stringResource(Res.string.lampcord_login_other_login_qr))
            }
        }
    }
}

@Composable
private fun ErrorBanner(message: String?) {
    if (message != null) {
        Spacer(modifier = Modifier.height(12.dp))
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.error,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
private fun PrimaryButton(text: String, enabled: Boolean, loading: Boolean, onClick: () -> Unit) {
    Spacer(modifier = Modifier.height(20.dp))
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier.fillMaxWidth().height(56.dp).handCursor(),
        shape = MaterialTheme.shapes.large
    ) {
        if (loading) {
            ContainedLoadingIndicator(
                modifier = Modifier.size(24.dp),
                containerColor = MaterialTheme.colorScheme.onPrimary,
                indicatorColor = MaterialTheme.colorScheme.primary,
            )
        } else {
            Text(text = text, style = MaterialTheme.typography.titleMedium)
        }
    }
}

@Composable
private fun OtherOptionsToggle(label: String, onToggle: () -> Unit) {
    Spacer(modifier = Modifier.height(4.dp))
    TextButton(onClick = onToggle, modifier = Modifier.handCursor()) {
        Text(label)
    }
}

@Composable
private fun LanguageOption(title: String, description: String, selected: Boolean, onSelect: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.medium)
            .clickable(onClick = onSelect)
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        RadioButton(selected = selected, onClick = null)
        Spacer(modifier = Modifier.width(12.dp))
        Column {
            Text(title, style = MaterialTheme.typography.bodyLarge)
            Text(
                description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun QrContent(
    state: RemoteAuthState,
    onRetry: () -> Unit
) {
    Surface(
        modifier = Modifier.size(200.dp),
        shape = MaterialTheme.shapes.large,
        color = Color.White,
        tonalElevation = 2.dp
    ) {
        Box(contentAlignment = Alignment.Center) {
            when (state) {
                is RemoteAuthState.QRReady -> {
                    val qrCode = remember(state.url) {
                        QRCode.ofSquares()
                            .withErrorCorrectionLevel(ErrorCorrectionLevel.MEDIUM)
                            .build(state.url)
                    }
                    Canvas(modifier = Modifier.fillMaxSize().padding(8.dp)) {
                        drawLocalQrCode(qrCode)
                    }
                }
                is RemoteAuthState.UserScanned -> {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        val avatarUrl = state.user.avatar?.let { CdnUrls.getUserAvatarUrl(state.user.id, it, 128) }
                        if (avatarUrl != null) {
                            me.lampu.lampcord.shared.ui.components.AsyncImage(
                                model = avatarUrl,
                                contentDescription = stringResource(Res.string.user_settings_avatar),
                                modifier = Modifier.size(64.dp).clip(CircleShape)
                            )
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            state.user.global_name ?: state.user.username,
                            style = MaterialTheme.typography.titleSmall,
                            color = MaterialTheme.colorScheme.onSurface
                        )
                        Text(
                            stringResource(Res.string.lampcord_login_qr_approve),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
                is RemoteAuthState.Error -> {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        modifier = Modifier.padding(16.dp)
                    ) {
                        Text(
                            stringResource(Res.string.error),
                            color = MaterialTheme.colorScheme.error,
                            style = MaterialTheme.typography.labelLarge
                        )
                        TextButton(onClick = onRetry, modifier = Modifier.handCursor()) {
                            Text(stringResource(Res.string.retry))
                        }
                    }
                }
                else -> ContainedLoadingIndicator(modifier = Modifier.size(32.dp))
            }
        }
    }
}

private fun verificationTokenFrom(input: String): String {
    val marker = "token="
    val index = input.indexOf(marker)
    return if (index >= 0) {
        input.substring(index + marker.length).substringBefore('&').substringBefore('#').trim()
    } else {
        input.trim()
    }
}

private fun DrawScope.drawLocalQrCode(qrCode: QRCode) {
    drawRect(color = Color.White)
    val quietZone = 4
    val moduleCount = qrCode.rawData.size
    val totalModules = moduleCount + quietZone * 2
    val moduleSize = min(size.width, size.height) / totalModules
    val codeSize = totalModules * moduleSize
    val offsetX = (size.width - codeSize) / 2f
    val offsetY = (size.height - codeSize) / 2f
    qrCode.rawData.forEach { cells ->
        cells.forEach { cell ->
            if (cell.dark) {
                drawRect(
                    color = Color.Black,
                    topLeft = Offset(
                        x = offsetX + (cell.col + quietZone) * moduleSize,
                        y = offsetY + (cell.row + quietZone) * moduleSize,
                    ),
                    size = Size(moduleSize, moduleSize),
                )
            }
        }
    }
}
