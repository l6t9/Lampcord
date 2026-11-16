package me.lampu.lampcord.shared.api

import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.client.request.header
import io.ktor.websocket.Frame
import io.ktor.websocket.readText
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.utils.CryptoUtils
import me.lampu.lampcord.shared.utils.Logging
import me.lampu.lampcord.shared.utils.RSAKeyPair
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlin.time.Duration.Companion.milliseconds

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

@OptIn(ExperimentalEncodingApi::class)
class RemoteAuthClient(
    private val httpClient: HttpClient,
    private val authApi: AuthApi
) {
    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = false
        explicitNulls = false
    }

    private val _state = MutableStateFlow<RemoteAuthState>(RemoteAuthState.Idle)
    val state: StateFlow<RemoteAuthState> = _state.asStateFlow()

    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())
    private var sessionJob: Job? = null
    private var session: DefaultClientWebSocketSession? = null
    private var heartbeatJob: Job? = null
    private var keyPair: RSAKeyPair? = null

    private var generation = 0

    private var exchanging = false

    fun start() {
        teardown()
        exchanging = false
        keyPair = CryptoUtils.generateRSAKeyPair()
        _state.value = RemoteAuthState.Connecting
        val gen = generation
        sessionJob = scope.launch { runSession(gen) }
    }

    fun reportFailure(message: String) {
        _state.value = RemoteAuthState.Error(message)
    }

    private suspend fun runSession(gen: Int) {
        try {
            httpClient.webSocket(
                urlString = "wss://remote-auth-gateway.discord.gg/?v=2",
                request = {
                    header("Origin", "https://discord.com")
                    header("Accept-Language", "en-US,en;q=0.9")
                    header("Cache-Control", "no-cache")
                    header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36")
                }
            ) {
                if (gen == generation) session = this
                for (frame in incoming) {
                    if (frame is Frame.Text) {
                        val text = frame.readText()
                        try {
                            handlePayload(json.decodeFromString<RemoteAuthPayload>(text))
                        } catch (e: CancellationException) {
                            throw e
                        } catch (e: Exception) {
                            Logging.e("RemoteAuth", "Error handling payload: ${e.message}")
                        }
                    }
                }
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            Logging.e("RemoteAuth", "Connection error: ${e.message}")
            if (gen == generation && _state.value !is RemoteAuthState.Finished) {
                _state.value = RemoteAuthState.Error(e.message ?: "Unknown error")
            }
        } finally {
            if (gen == generation) {
                heartbeatJob?.cancel()
                heartbeatJob = null
                session = null
                settleAfterClose()
            }
        }
    }

    private fun settleAfterClose() {
        if (exchanging) return
        val current = _state.value
        if (current is RemoteAuthState.Finished || current is RemoteAuthState.Error) return
        Logging.e("RemoteAuth", "Gateway closed before login finished (QR code expired, timed out, or handshake failed)")
        _state.value = RemoteAuthState.Error("This QR code expired or could not connect. Try again.")
    }

    private suspend fun handlePayload(payload: RemoteAuthPayload) {
        when (payload.op) {
            "hello" -> {
                payload.heartbeat_interval?.let { startHeartbeat(it) }
                sendPayload(RemoteAuthPayload(op = "init", encoded_public_key = keyPair?.getPublicKeyBase64()))
            }
            "nonce_proof" -> {
                val encryptedNonceB64 = payload.encrypted_nonce ?: return
                val decryptedNonce = keyPair?.decrypt(Base64.decode(encryptedNonceB64)) ?: return
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
                val decryptedUser = keyPair?.decrypt(Base64.decode(encryptedUserB64)) ?: return
                val parts = decryptedUser.decodeToString().split(":")
                if (parts.size >= 4) {
                    val user = RemoteUserPayload(
                        id = parts[0],
                        discriminator = parts[1],
                        avatar = parts[2].ifEmpty { null }?.takeIf { it != "0" },
                        username = parts[3]
                    )
                    _state.value = RemoteAuthState.UserScanned(user)
                }
            }
            "pending_login" -> {
                val ticket = payload.ticket ?: return
                exchanging = true
                scope.launch {
                    try {
                        exchangeTicket(ticket)
                    } finally {
                        exchanging = false
                    }
                }
            }
            "finish" -> {
                val encryptedTokenB64 = payload.encrypted_token ?: return
                val token = decryptToken(encryptedTokenB64)
                _state.value = if (token != null) {
                    RemoteAuthState.Finished(token)
                } else {
                    RemoteAuthState.Error("Failed to decrypt token")
                }
            }
            "cancel" -> {
                heartbeatJob?.cancel()
                _state.value = RemoteAuthState.Error("Canceled by user")
            }
        }
    }

    private suspend fun exchangeTicket(ticket: String) {
        val encryptedToken = authApi.exchangeRemoteAuthTicket(ticket)
        if (encryptedToken == null) {
            Logging.e("RemoteAuth", "Ticket exchange returned no token")
            _state.value = RemoteAuthState.Error("Failed to exchange ticket")
            return
        }
        val token = decryptToken(encryptedToken)
        if (token == null) {
            _state.value = RemoteAuthState.Error("Failed to decrypt token")
            return
        }
        _state.value = RemoteAuthState.Finished(token)
    }

    private fun decryptToken(encryptedTokenB64: String): String? = try {
        keyPair?.decrypt(Base64.decode(encryptedTokenB64))?.decodeToString()
    } catch (e: Exception) {
        Logging.e("RemoteAuth", "Token decrypt failed: ${e.message}")
        null
    }

    private fun startHeartbeat(interval: Long) {
        heartbeatJob?.cancel()
        heartbeatJob = scope.launch {
            while (isActive) {
                delay(interval.milliseconds)
                sendPayload(RemoteAuthPayload(op = "heartbeat"))
            }
        }
    }

    private suspend fun sendPayload(payload: RemoteAuthPayload) {
        val text = json.encodeToString(payload)
        session?.send(Frame.Text(text))
    }

    fun stop() {
        teardown()
        if (!exchanging && _state.value !is RemoteAuthState.Finished && _state.value !is RemoteAuthState.Error) {
            _state.value = RemoteAuthState.Idle
        }
    }

    private fun teardown() {
        generation++
        sessionJob?.cancel()
        sessionJob = null
        heartbeatJob?.cancel()
        heartbeatJob = null
        session = null
    }
}
