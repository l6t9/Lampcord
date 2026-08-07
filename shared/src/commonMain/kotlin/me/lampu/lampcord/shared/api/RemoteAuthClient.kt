package me.lampu.lampcord.shared.api

import io.ktor.client.*
import io.ktor.client.plugins.websocket.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.websocket.*
import me.lampu.lampcord.shared.utils.CryptoUtils
import me.lampu.lampcord.shared.utils.RSAKeyPair
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

@Serializable
data class RemoteAuthPayload(
    val op: String,
    val timeout_ms: Long? = null,
    val heartbeat_interval: Long? = null,
    val encoded_public_key: String? = null,
    val encrypted_nonce: String? = null,
    val nonce: String? = null,
    val fingerprint: String? = null,
    val encrypted_user_payload: String? = null,
    val user: RemoteUserPayload? = null,
    val ticket: String? = null,
    val encrypted_token: String? = null
)

@Serializable
data class RemoteUserPayload(
    val id: String,
    val username: String,
    val discriminator: String,
    val avatar: String?,
    val global_name: String? = null
)

sealed class RemoteAuthState {
    object Idle : RemoteAuthState()
    object Connecting : RemoteAuthState()
    data class QRReady(val url: String) : RemoteAuthState()
    data class UserScanned(val user: RemoteUserPayload) : RemoteAuthState()
    data class Finished(val token: String) : RemoteAuthState()
    data class Error(val message: String) : RemoteAuthState()
}

class RemoteAuthClient(
    private val httpClient: HttpClient,
    private val discordClient: DiscordClient
) {
    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = false
        explicitNulls = false // Crucial: Discord gateway hates null fields
    }

    private val _state = MutableStateFlow<RemoteAuthState>(RemoteAuthState.Idle)
    val state: StateFlow<RemoteAuthState> = _state.asStateFlow()

    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())
    private var session: DefaultClientWebSocketSession? = null
    private var heartbeatJob: Job? = null
    private var keyPair: RSAKeyPair? = null

    @OptIn(ExperimentalEncodingApi::class, ExperimentalStdlibApi::class)
    suspend fun start() {
        _state.value = RemoteAuthState.Connecting
        keyPair = CryptoUtils.generateRSAKeyPair()

        try {
            httpClient.webSocket(
                urlString = "wss://remote-auth-gateway.discord.gg/?v=2",
                request = {
                    header("Origin", "https://discord.com")
                    header("Accept-Language", "en-US,en;q=0.9")
                    header("Cache-Control", "no-cache")
                    // We don't have easy access to getPlatformName here without making it more complex,
                    // so we'll use a standard Discord Desktop-like UA which is safe for Remote Auth.
                    header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36")
                }
            ) {
                session = this
                
                for (frame in incoming) {
                    if (frame is Frame.Text) {
                        val text = frame.readText()
                        try {
                            val payload = json.decodeFromString<RemoteAuthPayload>(text)
                            handlePayload(payload)
                        } catch (e: Exception) {
                            println("RemoteAuth: Error decoding payload: ${e.message}")
                        }
                    }
                }
            }
        } catch (e: Exception) {
            println("RemoteAuth Error: ${e.message}")
            _state.value = RemoteAuthState.Error(e.message ?: "Unknown error")
        } finally {
            stop()
        }
    }

    private suspend fun handlePayload(payload: RemoteAuthPayload) {
        when (payload.op) {
            "hello" -> {
                payload.heartbeat_interval?.let { startHeartbeat(it) }
                sendPayload(RemoteAuthPayload(op = "init", encoded_public_key = keyPair?.getPublicKeyBase64()))
            }
            "nonce_proof" -> {
                val encryptedNonceB64 = payload.encrypted_nonce ?: return
                val encryptedNonce = Base64.Default.decode(encryptedNonceB64)
                val decryptedNonce = keyPair?.decrypt(encryptedNonce) ?: return
                val nonce = Base64.UrlSafe.encode(decryptedNonce).replace("=", "")
                sendPayload(RemoteAuthPayload(op = "nonce_proof", nonce = nonce))
            }
            "pending_remote_init" -> {
                payload.fingerprint?.let {
                    _state.value = RemoteAuthState.QRReady("https://discord.com/ra/$it")
                }
            }
            "pending_ticket" -> {
                val encryptedUserB64 = payload.encrypted_user_payload ?: return
                val encryptedUser = Base64.Default.decode(encryptedUserB64)
                val decryptedUser = keyPair?.decrypt(encryptedUser) ?: return
                val userStr = decryptedUser.decodeToString()
                val parts = userStr.split(":")
                if (parts.size >= 4) {
                    val user = RemoteUserPayload(
                        id = parts[0],
                        discriminator = parts[1],
                        avatar = if (parts[2].isEmpty()) null else parts[2],
                        username = parts[3]
                    )
                    _state.value = RemoteAuthState.UserScanned(user)
                }
            }
            "pending_login" -> {
                payload.ticket?.let { ticket ->
                    scope.launch {
                        val encryptedToken = discordClient.exchangeRemoteAuthTicket(ticket)
                        if (encryptedToken != null) {
                            try {
                                val encryptedData = Base64.Default.decode(encryptedToken)
                                val decryptedToken = keyPair?.decrypt(encryptedData) ?: return@launch
                                val token = decryptedToken.decodeToString()
                                _state.value = RemoteAuthState.Finished(token)
                            } catch (e: Exception) {
                                _state.value = RemoteAuthState.Error("Failed to decrypt token")
                            }
                        } else {
                            _state.value = RemoteAuthState.Error("Failed to exchange ticket")
                        }
                    }
                }
            }
            "finish" -> {
                val encryptedTokenB64 = payload.encrypted_token ?: return
                val encryptedToken = Base64.Default.decode(encryptedTokenB64)
                val decryptedToken = keyPair?.decrypt(encryptedToken) ?: return
                val token = decryptedToken.decodeToString()
                _state.value = RemoteAuthState.Finished(token)
            }
            "cancel" -> {
                _state.value = RemoteAuthState.Error("Canceled by user")
                stop()
            }
        }
    }

    private fun startHeartbeat(interval: Long) {
        heartbeatJob?.cancel()
        heartbeatJob = CoroutineScope(Dispatchers.Default).launch {
            while (isActive) {
                delay(interval)
                sendPayload(RemoteAuthPayload(op = "heartbeat"))
            }
        }
    }

    private suspend fun sendPayload(payload: RemoteAuthPayload) {
        val text = json.encodeToString(payload)
        session?.send(Frame.Text(text))
    }

    fun stop() {
        heartbeatJob?.cancel()
        heartbeatJob = null
        session = null
        if (_state.value !is RemoteAuthState.Finished && _state.value !is RemoteAuthState.Error) {
            _state.value = RemoteAuthState.Idle
        }
    }
}
