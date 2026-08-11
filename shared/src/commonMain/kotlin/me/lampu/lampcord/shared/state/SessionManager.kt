package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.model.LoginRequest
import me.lampu.lampcord.shared.model.LoginResponse
import me.lampu.lampcord.shared.model.MFALoginRequest
import me.lampu.lampcord.shared.settings.Settings

class SessionManager(
    val gatewayManager: GatewayManager,
    val voiceGatewayManager: VoiceGatewayManager,
    val discordClient: DiscordClient,
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
    val commandStore: CommandStore
) {
    private val scope = CoroutineScope(Dispatchers.Main)

    init {
        scope.launch {
            gatewayManager.events.collect { gatewayHandler.handleGatewayEvent(it) }
        }

        val savedToken = Settings.shared.discordToken
        if (savedToken.isNotBlank()) connect(savedToken)
    }

    fun connect(token: String) {
        discordClient.setToken(token)
        Settings.shared.discordToken = token
        navigationStore.isConnecting = true
        gatewayManager.connect(token)
    }

    fun disconnect() {
        gatewayManager.disconnect()
        voiceGatewayManager.disconnect()
        navigationStore.isConnected = false
        navigationStore.isConnecting = false
        discordClient.setToken(null)
        Settings.shared.discordToken = ""
        clearAllStores()
    }

    fun switchAccount(token: String) {
        disconnect()
        connect(token)
    }

    fun logout(token: String) {
        tokenStore.removeAccount(token)
        if (Settings.shared.discordToken == token) {
            disconnect()
        }
    }

    private fun clearAllStores() {
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
    }

    suspend fun login(email: String, pass: String): LoginResponse? {
        val fingerprint = discordClient.getFingerprint() ?: ""
        return discordClient.login(LoginRequest(email, pass), fingerprint)
    }

    suspend fun verifyMFA(ticket: String, code: String, type: String): Boolean {
        val fingerprint = discordClient.getFingerprint() ?: ""
        val res = discordClient.loginMFA(MFALoginRequest(ticket, code), fingerprint, type)
        return if (res?.token != null) {
            connect(res.token)
            true
        } else false
    }
}
