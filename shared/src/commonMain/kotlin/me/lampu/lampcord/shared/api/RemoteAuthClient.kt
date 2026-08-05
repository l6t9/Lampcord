package me.lampu.lampcord.shared.api

import io.ktor.client.*
import io.ktor.client.plugins.websocket.*
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
    val public_key: String? = null,
    val encrypted_nonce: String? = null,
    val proof: String? = null,
    val fingerprint: String? = null,
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
    private val json: Json
) {
    private val _state = MutableStateFlow<RemoteAuthState>(RemoteAuthState.Idle)
    val state: StateFlow<RemoteAuthState> = _state.asStateFlow()

    private var session: DefaultClientWebSocketSession? = null
    private var heartbeatJob: Job? = null
    private var keyPair: RSAKeyPair? = null

    @OptIn(ExperimentalEncodingApi::class, ExperimentalStdlibApi::class)
    suspend fun start() {
        _state.value = RemoteAuthState.Connecting
        keyPair = CryptoUtils.generateRSAKeyPair()

        try {
            httpClient.webSocket(
                method = HttpMethod.Get,
                host = "remote_auth-gateway.discord.gg",
                path = "/?v=2"
            ) {
                session = this
                
                for (frame in incoming) {
                    if (frame is Frame.Text) {
                        val text = frame.readText()
                        val payload = json.decodeFromString<RemoteAuthPayload>(text)
                        handlePayload(payload)
                    }
                }
            }
        } catch (e: Exception) {
            _state.value = RemoteAuthState.Error(e.message ?: "Unknown error")
        } finally {
            stop()
        }
    }

    private suspend fun handlePayload(payload: RemoteAuthPayload) {
        when (payload.op) {
            "hello" -> {
                payload.heartbeat_interval?.let { startHeartbeat(it) }
                sendPayload(RemoteAuthPayload(op = "init", public_key = keyPair?.getPublicKeyBase64()))
            }
            "nonce_proof" -> {
                val encryptedNonceB64 = payload.encrypted_nonce ?: return
                val encryptedNonce = Base64.Default.decode(encryptedNonceB64)
                val decryptedNonce = keyPair?.decrypt(encryptedNonce) ?: return
                val hash = CryptoUtils.sha256(decryptedNonce)
                val proof = Base64.UrlSafe.encode(hash).replace("=", "")
                sendPayload(RemoteAuthPayload(op = "nonce_proof", proof = proof))
            }
            "pending_remote_init" -> {
                payload.fingerprint?.let {
                    _state.value = RemoteAuthState.QRReady("https://discord.com/ra/$it")
                }
            }
            "pending_ticket" -> {
                payload.user?.let {
                    _state.value = RemoteAuthState.UserScanned(it)
                }
            }
            "pending_login" -> {
                // ticket is in payload.ticket
                // We should wait for finish or do the POST call?
                // Actually v2 finish payload has the encrypted token.
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
