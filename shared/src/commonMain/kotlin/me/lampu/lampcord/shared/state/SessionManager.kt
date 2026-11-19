package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.api.AuthApi
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.api.ForgotPasswordResult
import me.lampu.lampcord.shared.model.LoginRequest
import me.lampu.lampcord.shared.model.LoginResponse
import me.lampu.lampcord.shared.model.MFALoginRequest
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.notifications.PushTokenRegistrar
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.update.UpdateManager

class SessionManager(
    val gatewayManager: GatewayManager,
    val voiceGatewayManager: VoiceGatewayManager,
    val authApi: AuthApi,
    val navigationStore: NavigationStore,
    val tokenStore: TokenStore,
    val userStore: UserStore,
    val gatewayHandler: GatewayHandler,
    val entityStore: EntityStore,
    val readStateStore: ReadStateStore,
    val userGuildSettingsStore: UserGuildSettingsStore,
    val presenceStore: PresenceStore,
    val relationshipStore: RelationshipStore,
    val guildStore: GuildStore,
    val memberListStore: MemberListStore,
    val messageStore: MessageStore,
    val typingStore: TypingStore,
    val commandStore: CommandStore,
    val voiceStore: VoiceStore,
    val updateManager: UpdateManager,
    val themeStore: ThemeStore,
    val pushTokenRegistrar: PushTokenRegistrar? = null,
    private val notifier: MessageNotifier? = null
) {
    private val scope = CoroutineScope(
        SupervisorJob() +
            Dispatchers.Main +
            CoroutineExceptionHandler { _, throwable ->
                me.lampu.lampcord.shared.utils.Logging.e(
                    tag = "SessionManager",
                    message = "Uncaught exception in session scope",
                    throwable = throwable
                )
            }
    )

    private var loginFingerprint: String? = null

    init {
        scope.launch {

            gatewayManager.events.collect { payload ->
                try {
                    if (payload.op == -1 && payload.t == "AUTH_FAILED") {
                        logout(Settings.shared.discordToken)
                    } else {
                        withContext(Dispatchers.Default) {
                            gatewayHandler.handleGatewayEvent(payload)
                        }
                    }
                } catch (e: kotlin.coroutines.cancellation.CancellationException) {
                    throw e
                } catch (e: Exception) {
                    me.lampu.lampcord.shared.utils.Logging.e(
                        tag = "SessionManager",
                        message = "Failed to handle gateway event ${payload.op}:${payload.t}",
                        throwable = e
                    )
                }
            }
        }

        scope.launch {
            authApi.rest.unauthorizedEvents.collect {
                logout(Settings.shared.discordToken)
            }
        }

        val savedToken = Settings.shared.discordToken
        if (savedToken.isNotBlank()) connect(savedToken)
    }

    fun connect(token: String) {
        authApi.setToken(token)
        Settings.shared.discordToken = token
        loginFingerprint = null
        navigationStore.isConnecting = true
        gatewayManager.connect(token)
        pushTokenRegistrar?.register()
    }

    fun disconnect() {
        // Must run before the session token is cleared so the DELETE carries valid auth.
        pushTokenRegistrar?.unregister()
        notifier?.dismissAllNotifications()
        voiceStore.disconnectFromVoice()
        gatewayManager.disconnect()
        navigationStore.isConnected = false
        navigationStore.isConnecting = false
        authApi.setToken(null)
        Settings.shared.discordToken = ""
        clearAllStores()
    }

    fun shutdown() {
        disconnect()
        gatewayManager.close()
        voiceGatewayManager.close()
        updateManager.close()
        themeStore.close()
        authApi.rest.closeClient()
        scope.cancel()
    }

    fun signOut() {
        val token = Settings.shared.discordToken.trim()
        if (token.isEmpty()) {
            disconnect()
            return
        }
        scope.launch { authApi.logoutRemote(token) }
        removeActiveAccount(token)
    }

    fun switchAccount(token: String) {
        disconnect()
        connect(token)
    }

    fun logout(token: String) {
        val cleanToken = token.trim()
        if (Settings.shared.discordToken.trim() == cleanToken) {
            removeActiveAccount(cleanToken)
        } else {
            tokenStore.removeAccount(cleanToken)
        }
    }

    private fun removeActiveAccount(token: String) {
        val accounts = tokenStore.getAccounts()
        val index = accounts.indexOfFirst { it.token == token }.coerceAtLeast(0)
        tokenStore.removeAccount(token)
        val remaining = tokenStore.getAccounts()
        if (remaining.isEmpty()) {
            disconnect()
        } else {
            switchAccount(remaining[index.coerceAtMost(remaining.lastIndex)].token)
        }
    }

    private fun clearAllStores() {
        gatewayManager.clearSubscriptions()
        navigationStore.clear()
        entityStore.clear()
        readStateStore.clear()
        userGuildSettingsStore.clear()
        presenceStore.clear()
        userStore.clear()
        relationshipStore.clear()
        guildStore.clear()
        memberListStore.clear()
        messageStore.clear()
        typingStore.clear()
        commandStore.clear()
        voiceStore.clear()
    }

    private suspend fun loginFlowFingerprint(): String {
        loginFingerprint?.takeIf { it.isNotEmpty() }?.let { return it }
        val fingerprint = authApi.getFingerprint() ?: ""
        loginFingerprint = fingerprint
        return fingerprint
    }

    suspend fun login(email: String, pass: String, undelete: Boolean = false): LoginResponse? {
        val fingerprint = loginFlowFingerprint()
        return authApi.login(LoginRequest(login = email, password = pass, undelete = undelete), fingerprint)
    }

    suspend fun sendMfaSms(ticket: String): Result<String> =
        authApi.sendMfaSms(ticket, loginFlowFingerprint())

    suspend fun verifyLoginPhone(phone: String, code: String): Result<String> =
        authApi.verifyPhoneForLogin(phone, code, loginFlowFingerprint())

    suspend fun resendLoginPhoneCode(phone: String): Result<Unit> =
        authApi.resendPhoneCode(phone, loginFlowFingerprint())

    suspend fun authorizeLoginIp(token: String): Result<Unit> =
        authApi.authorizeIp(token, loginFlowFingerprint())

    suspend fun forgotPassword(login: String): ForgotPasswordResult =
        authApi.forgotPassword(login, loginFlowFingerprint())

    suspend fun verifyMFA(
        code: String,
        ticket: String,
        type: String,
        loginInstanceId: String? = null
    ): LoginResponse? {
        val fingerprint = loginFlowFingerprint()
        val request = MFALoginRequest(code = code, ticket = ticket, login_instance_id = loginInstanceId)
        return authApi.loginMFA(request, fingerprint, type)
    }
}
