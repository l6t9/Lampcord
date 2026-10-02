package me.lampu.lampcord.shared.gateway

import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.voice.*
import me.lampu.lampcord.shared.settings.Settings
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.util.concurrent.ConcurrentHashMap

actual class VoiceGatewayManager actual constructor(private val client: HttpClient, private val json: Json) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val mutableStatus = MutableStateFlow(VoiceConnectionStatus())
    actual val status: StateFlow<VoiceConnectionStatus> = mutableStatus.asStateFlow()
    @Volatile private var active: Connection? = null
    private var job: Job? = null
    private var muted = false
    private var deafened = false
    private var speaker = false

    actual fun connect(endpoint: String, serverId: String, channelId: String, userId: String, sessionId: String, token: String) {
        val previousJob = job
        disconnect()
        val connection = Connection(endpoint, serverId, channelId, userId, sessionId, token)
        connection.muted = muted
        connection.deafened = deafened
        active = connection
        mutableStatus.value = VoiceConnectionStatus(channelId, VoicePhase.CONNECTING)
        job = scope.launch {
            try {
                previousJob?.join()
                connection.run()
                error("Voice server closed the connection")
            } catch (e: CancellationException) {
                throw e
            } catch (e: Throwable) {
                if (active === connection) {
                    // Protocol payloads and tokens must never enter logs or the error UI.
                    val message = if (e is VoiceFailure) e.message else "Voice connection failed. Check your audio devices and network, then retry."
                    mutableStatus.value = VoiceConnectionStatus(channelId, VoicePhase.FAILED, message)
                }
            } finally {
                connection.close()
            }
        }
    }

    actual fun setMuted(muted: Boolean, deafened: Boolean) {
        this.muted = muted
        this.deafened = deafened
        active?.let { it.muted = muted; it.deafened = deafened }
    }

    actual fun setSpeaker(enabled: Boolean) {
        speaker = enabled
        active?.audio?.setSpeaker(enabled)
    }

    actual fun disconnect() {
        val old = active
        active = null
        old?.stopIO()
        job?.cancel()
        job = null
        mutableStatus.value = VoiceConnectionStatus()
    }

    actual fun close() {
        disconnect()
        scope.cancel()
    }

    private class VoiceFailure(message: String) : Exception(message)

    private inner class Connection(
        val endpoint: String, val serverId: String, val channelId: String,
        val userId: String, val sessionId: String, val token: String
    ) {
        @Volatile var muted = false
        @Volatile var deafened = false
        @Volatile var audio: VoiceAudio? = null
        @Volatile private var socket: DatagramSocket? = null
        private var handle = 0L
        private val lock = Any()
        private var packets: VoicePackets? = null
        @Volatile private var ws: DefaultClientWebSocketSession? = null
        private var ssrc = 0
        private var mode = ""
        private var sequence = -1
        private var acknowledged = true
        @Volatile private var secure = false
        @Volatile private var gatewayReady = false
        private var verificationCode = ""
        private val users = mutableSetOf(userId)
        private val sources = ConcurrentHashMap<Int, String>()
        private val playout = mutableMapOf<String, VoicePlayout>()
        private val speaking = ConcurrentHashMap<String, Long>()
        private var mediaStarted = false
        private var resetCount = 0

        private fun update(phase: VoicePhase = if (!gatewayReady) VoicePhase.CONNECTING else if (secure) VoicePhase.SECURE else VoicePhase.CONNECTED) {
            if (active === this) mutableStatus.value = VoiceConnectionStatus(channelId, phase, verificationCode = verificationCode, speaking = speaking.keys.toSet())
        }

        suspend fun run() = coroutineScope {
            require(serverId.toULongOrNull() != null && channelId.toULongOrNull() != null && userId.toULongOrNull() != null)
            require(endpoint.matches(Regex("[A-Za-z0-9.-]+(?::[0-9]{1,5})?")))
            synchronized(lock) { handle = NativeVoice.create(userId, channelId); check(handle != 0L) }
            try {
            val handshake = launch {
                delay(30_000)
                if (!mediaStarted) throw VoiceFailure("Voice handshake timed out. Retry the call.")
            }
            val mediaScope = this
            var attempts = 0
            while (isActive) {
                val resumeTimeout = if (mediaStarted) launch {
                    delay(15_000)
                    if (!gatewayReady) throw VoiceFailure("Voice reconnection timed out. Rejoin the call.")
                } else null
                try {
                    client.webSocket("wss://${endpoint.removeSuffix(":80")}/?v=8") {
                        ws = this
                        var heartbeat: Job? = null
                        try {
                            for (frame in incoming) {
                                if (frame.data.size > 1024 * 1024) throw VoiceFailure("Oversized voice gateway message")
                                when (frame) {
                                    is Frame.Binary -> binary(frame.data)
                                    is Frame.Text -> {
                                        val payload = json.parseToJsonElement(frame.readText()).jsonObject
                                        payload["seq"]?.jsonPrimitive?.intOrNull?.let { sequence = it }
                                        val data = payload["d"] as? JsonObject ?: JsonObject(emptyMap())
                                        when (payload.getValue("op").jsonPrimitive.int) {
                                            8 -> {
                                                val interval = data.getValue("heartbeat_interval").jsonPrimitive.double.toLong().coerceIn(1000, 60_000)
                                                acknowledged = true
                                                heartbeat?.cancel()
                                                heartbeat = launch {
                                                    while (isActive) {
                                                        if (!acknowledged) throw VoiceFailure("Voice heartbeat timed out")
                                                        acknowledged = false
                                                        send(3, buildJsonObject { put("t", System.currentTimeMillis()); put("seq_ack", sequence) })
                                                        delay(interval)
                                                    }
                                                }
                                                send(if (mediaStarted) 7 else 0, buildJsonObject {
                                                    put("server_id", serverId); put("user_id", userId)
                                                    put("session_id", sessionId); put("token", token)
                                                    if (mediaStarted) put("seq_ack", sequence) else {
                                                        put("video", false); put("max_dave_protocol_version", 1)
                                                    }
                                                })
                                            }
                                            2 -> ready(data)
                                            4 -> {
                                                if (mediaStarted) throw VoiceFailure("Unexpected voice session replacement")
                                                requireE2EE(data["dave_protocol_version"]?.jsonPrimitive?.intOrNull ?: 0)
                                                if (data["mode"]?.jsonPrimitive?.content != mode) throw VoiceFailure("Voice encryption mode mismatch")
                                                val key = data.getValue("secret_key").jsonArray.map {
                                                    it.jsonPrimitive.int.also { value -> require(value in 0..255) }.toByte()
                                                }.toByteArray()
                                                synchronized(lock) { packets = VoicePackets(key, mode == AES_MODE, ssrc) }
                                                initializeDave()
                                                audio = createVoiceAudio().also { it.setSpeaker(speaker) }
                                                mediaStarted = true
                                                gatewayReady = true
                                                handshake.cancel()
                                                update()
                                                mediaScope.launch { receiveAudio() }
                                                mediaScope.launch { captureAudio() }
                                                mediaScope.launch { playAudio() }
                                                mediaScope.launch { keepAlive() }
                                            }
                                            5 -> {
                                                val user = data.getValue("user_id").jsonPrimitive.content
                                                val source = data.getValue("ssrc").jsonPrimitive.content.toUInt().toInt()
                                                if (sources.size < 1000 || sources.containsKey(source)) sources[source] = user
                                            }
                                            6 -> acknowledged = true
                                            9 -> { attempts = 0; gatewayReady = true; update() }
                                            11 -> {
                                                val ids = data.getValue("user_ids").jsonArray.map { it.jsonPrimitive.content }
                                                require(users.size + ids.size <= 10000)
                                                users.addAll(ids)
                                            }
                                            13 -> {
                                                val user = data.getValue("user_id").jsonPrimitive.content
                                                users.remove(user); speaking.remove(user)
                                                synchronized(lock) {
                                                    sources.entries.filter { it.value == user }.forEach { packets?.forget(it.key); sources.remove(it.key) }
                                                    playout.remove(user)
                                                    NativeVoice.removeUser(handle, user)
                                                }
                                                update()
                                            }
                                            21 -> {
                                                requireE2EE(data.getValue("protocol_version").jsonPrimitive.int)
                                                val id = data.getValue("transition_id").jsonPrimitive.int
                                                synchronized(lock) { NativeVoice.transition(handle, id, true) }
                                                transitionReady(id)
                                            }
                                            22 -> {
                                                val id = data.getValue("transition_id").jsonPrimitive.int
                                                synchronized(lock) { NativeVoice.transition(handle, id, false) }
                                                markSecure()
                                            }
                                            24 -> {
                                                requireE2EE(data.getValue("protocol_version").jsonPrimitive.int)
                                                if (data.getValue("epoch").jsonPrimitive.content == "1") initializeDave()
                                            }
                                        }
                                    }
                                    else -> Unit
                                }
                            }
                            val reason = closeReason.await()
                            if (reason?.code?.toInt() in listOf(4004, 4006, 4009, 4011, 4012, 4014, 4016, 4017)) {
                                throw VoiceFailure("Voice session ended (${reason?.code}). Retry to rejoin.")
                            }
                            error("Voice WebSocket closed")
                        } finally {
                            heartbeat?.cancel()
                            gatewayReady = false
                            ws = null
                        }
                    }
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    if (!mediaStarted || e is VoiceFailure || ++attempts > 2) throw e
                    update(VoicePhase.CONNECTING)
                    delay(attempts * 1000L)
                } finally {
                    resumeTimeout?.cancel()
                }
            }
            } finally {
                stopIO()
            }
        }

        private fun requireE2EE(version: Int) {
            if (version != 1) {
                secure = false
                throw VoiceFailure("This call did not negotiate supported DAVE encryption. Microphone transmission has been stopped.")
            }
        }

        private suspend fun ready(data: JsonObject) {
            if (socket != null) throw VoiceFailure("Unexpected voice ready message")
            ssrc = data.getValue("ssrc").jsonPrimitive.content.toUInt().toInt()
            val modes = data.getValue("modes").jsonArray.map { it.jsonPrimitive.content }
            mode = when { AES_MODE in modes -> AES_MODE; XCHACHA_MODE in modes -> XCHACHA_MODE; else -> throw VoiceFailure("No supported voice transport encryption") }
            val port = data.getValue("port").jsonPrimitive.int.also { require(it in 1..65535) }
            val udp = DatagramSocket().also { socket = it; it.soTimeout = 5000 }
            udp.connect(InetSocketAddress(data.getValue("ip").jsonPrimitive.content, port))
            val request = ByteBuffer.allocate(74).putShort(1).putShort(70).putInt(ssrc).array()
            val response = ByteArray(256)
            var discovered: DatagramPacket? = null
            repeat(3) {
                if (discovered == null) {
                    udp.send(DatagramPacket(request, request.size))
                    val packet = DatagramPacket(response, response.size)
                    try {
                        udp.receive(packet)
                        if (packet.length == 74 && unsignedShort(response, 0) == 2 && unsignedShort(response, 2) == 70 && ByteBuffer.wrap(response, 4, 4).int == ssrc) discovered = packet
                    } catch (_: SocketTimeoutException) { }
                }
            }
            if (discovered == null) throw VoiceFailure("Voice UDP discovery failed. Check your network or VPN.")
            val end = (8 until 72).firstOrNull { response[it] == 0.toByte() } ?: throw VoiceFailure("Invalid voice discovery response")
            val address = response.copyOfRange(8, end).toString(Charsets.US_ASCII)
            require(address.matches(Regex("[0-9a-fA-F:.]+")))
            val externalPort = unsignedShort(response, 72).also { require(it != 0) }
            send(1, buildJsonObject {
                put("protocol", "udp")
                putJsonObject("data") { put("address", address); put("port", externalPort); put("mode", mode) }
                putJsonArray("codecs") {
                    add(buildJsonObject { put("name", "opus"); put("type", "audio"); put("priority", 1000); put("payload_type", 120) })
                }
            })
        }

        private suspend fun initializeDave() {
            secure = false
            verificationCode = ""
            val keyPackage = synchronized(lock) { NativeVoice.init(handle, 1) }
            binarySend(26, keyPackage)
            if (mediaStarted) update()
        }

        private suspend fun binary(bytes: ByteArray) {
            require(bytes.size >= 3)
            sequence = unsignedShort(bytes, 0)
            val op = bytes[2].toInt() and 255
            val body = bytes.copyOfRange(3, bytes.size)
            when (op) {
                25 -> synchronized(lock) { NativeVoice.externalSender(handle, body) }
                27 -> {
                    val result = synchronized(lock) { NativeVoice.proposals(handle, body, users.toTypedArray()) }
                    if (result != null) binarySend(28, result)
                }
                29, 30 -> {
                    require(body.size >= 2)
                    val transition = unsignedShort(body, 0)
                    val result = synchronized(lock) { NativeVoice.commit(handle, body.copyOfRange(2, body.size), users.toTypedArray(), transition, op == 30) }
                    when (result) {
                        0 -> {
                            if (++resetCount > 3) throw VoiceFailure("DAVE key exchange failed repeatedly. Rejoin the call.")
                            send(31, buildJsonObject { put("transition_id", transition) })
                            initializeDave()
                        }
                        1, 2 -> { resetCount = 0; transitionReady(transition) }
                    }
                }
            }
        }

        private suspend fun transitionReady(id: Int) {
            if (id == 0) markSecure() else send(23, buildJsonObject { put("transition_id", id) })
        }

        private fun markSecure() {
            secure = true
            verificationCode = synchronized(lock) { NativeVoice.authenticator(handle) }.joinToString("") { "%02x".format(it.toInt() and 255) }.chunked(8).joinToString(" ")
            update()
        }

        private suspend fun send(op: Int, data: JsonObject) {
            checkNotNull(ws).send(json.encodeToString(buildJsonObject { put("op", op); put("d", data) }))
        }
        private suspend fun binarySend(op: Int, body: ByteArray) { checkNotNull(ws).send(Frame.Binary(true, byteArrayOf(op.toByte()) + body)) }

        private suspend fun sendSpeaking(value: Int): Boolean = try {
            send(5, buildJsonObject { put("speaking", value); put("delay", 0); put("ssrc", ssrc.toLong() and 0xffffffffL) })
            true
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
            false // A closing WebSocket must not take down the independent audio/UDP session.
        }

        private suspend fun captureAudio() {
            val pcm = ShortArray(1920)
            var transmitting = false
            var silence = 0
            while (currentCoroutineContext().isActive) {
                checkNotNull(audio).read(pcm)
                val canSend = secure && !muted && !deafened && gatewayReady
                if (canSend) {
                    if (!transmitting) {
                        if (!sendSpeaking(1)) { pcm.fill(0); continue }
                        transmitting = true
                    }
                    silence = 5
                    val denoise = Settings.shared.noiseCancellation
                    val packet = synchronized(lock) {
                        NativeVoice.encrypt(handle, ssrc, NativeVoice.encode(handle, pcm, denoise))?.let { packets?.encrypt(it) }
                    }
                    packet?.let { socket?.send(DatagramPacket(it, it.size)) }
                    if (pcm.any { kotlin.math.abs(it.toInt()) > 600 }) speaking[userId] = System.nanoTime()
                } else if (transmitting && gatewayReady) {
                    if (silence-- > 0) {
                        val packet = synchronized(lock) { packets?.encrypt(OPUS_SILENCE) }
                        packet?.let { socket?.send(DatagramPacket(it, it.size)) }
                    } else {
                        if (sendSpeaking(0)) transmitting = false
                    }
                }
                pcm.fill(0)
            }
        }

        private suspend fun receiveAudio() {
            val buffer = ByteArray(65536)
            while (currentCoroutineContext().isActive) {
                val datagram = DatagramPacket(buffer, buffer.size)
                try { checkNotNull(socket).receive(datagram) } catch (_: SocketTimeoutException) { continue }
                val data = buffer.copyOf(datagram.length)
                synchronized(lock) {
                    val packet = packets?.decrypt(data) ?: return@synchronized
                    val user = sources[packet.ssrc] ?: return@synchronized
                    if (user == userId || deafened) return@synchronized
                    if (packet.frame.contentEquals(OPUS_SILENCE)) return@synchronized
                    val opus = NativeVoice.decrypt(handle, user, packet.frame) ?: return@synchronized
                    val now = System.nanoTime()
                    playout.getOrPut(user) { VoicePlayout() }.offer(packet.sequence, opus, now)
                    speaking[user] = now
                }
            }
        }

        private suspend fun playAudio() {
            val mixed = IntArray(1920)
            val pcm = ShortArray(1920)
            while (currentCoroutineContext().isActive) {
                mixed.fill(0)
                synchronized(lock) {
                    if (!deafened) playout.forEach { (user, buffer) -> buffer.mix(mixed, System.nanoTime()) { NativeVoice.decode(handle, user, it) } }
                    else playout.clear()
                }
                for (i in pcm.indices) pcm[i] = mixed[i].coerceIn(-32768, 32767).toShort()
                checkNotNull(audio).write(pcm)
            }
        }

        private suspend fun keepAlive() {
            var tick = 0
            while (currentCoroutineContext().isActive) {
                if (tick++ % 25 == 0) {
                    val packet = ByteBuffer.allocate(74).putShort(1).putShort(70).putInt(ssrc).array()
                    socket?.send(DatagramPacket(packet, packet.size))
                }
                val cutoff = System.nanoTime() - 300_000_000L
                val removed = speaking.entries.removeIf { it.value < cutoff }
                if (removed || tick % 5 == 0) update()
                delay(200)
            }
        }

        fun stopIO() {
            secure = false
            socket?.close()
            audio?.close()
        }

        fun close() {
            stopIO()
            synchronized(lock) {
                if (handle != 0L) NativeVoice.destroy(handle)
                handle = 0L
                packets?.close(); packets = null
                playout.clear()
            }
        }
    }
}
