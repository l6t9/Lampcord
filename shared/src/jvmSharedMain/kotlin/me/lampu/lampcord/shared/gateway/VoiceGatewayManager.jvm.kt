package me.lampu.lampcord.shared.gateway

import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeoutConfig
import io.ktor.client.plugins.timeout
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.webSocket
import androidx.compose.ui.graphics.ImageBitmap
import io.ktor.websocket.*
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.voice.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Logging
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress
import java.net.SocketTimeoutException
import java.nio.ByteBuffer
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

actual class VoiceGatewayManager actual constructor(private val client: HttpClient, private val json: Json) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val mutableStatus = MutableStateFlow(VoiceConnectionStatus())
    actual val status: StateFlow<VoiceConnectionStatus> = mutableStatus.asStateFlow()
    private val mutableVideo = MutableStateFlow<Map<String, ImageBitmap>>(emptyMap())
    actual val videoFrames: StateFlow<Map<String, ImageBitmap>> = mutableVideo.asStateFlow()
    private val mutableStreamStatus = MutableStateFlow(VoiceConnectionStatus())
    actual val streamStatus: StateFlow<VoiceConnectionStatus> = mutableStreamStatus.asStateFlow()
    private val mutableStreamFrame = MutableStateFlow<ImageBitmap?>(null)
    actual val streamFrame: StateFlow<ImageBitmap?> = mutableStreamFrame.asStateFlow()
    @Volatile private var active: Connection? = null
    @Volatile private var stream: Connection? = null
    private var job: Job? = null
    private var streamJob: Job? = null
    private var muted = false
    private var deafened = false
    private var speaker = false

    actual fun connect(endpoint: String, serverId: String, channelId: String, userId: String, sessionId: String, token: String) {
        val previousJob = job
        disconnect()
        val connection = Connection(endpoint, serverId, channelId, channelId, userId, sessionId, token, isStream = false)
        connection.muted = muted
        connection.deafened = deafened
        active = connection
        Logging.i("Voice", "Connecting channel=$channelId endpoint=$endpoint")
        mutableStatus.value = VoiceConnectionStatus(channelId, VoicePhase.CONNECTING)
        job = launchConnection(connection, previousJob, "Voice connection failed. Check your audio devices and network, then retry.")
    }

    actual fun watchStream(endpoint: String, rtcServerId: String, streamKey: String, userId: String, sessionId: String, token: String) {
        val previousJob = streamJob
        stopStream()
        // Go Live connections run their own MLS group, keyed one below the RTC server ID.
        val group = rtcServerId.toULongOrNull()?.takeIf { it > 0u }?.minus(1u)?.toString() ?: return
        val connection = Connection(endpoint, rtcServerId, streamKey, group, userId, sessionId, token, isStream = true)
        connection.muted = true
        connection.deafened = deafened
        stream = connection
        Logging.i("Voice", "Watching stream=$streamKey endpoint=$endpoint")
        mutableStreamStatus.value = VoiceConnectionStatus(streamKey, VoicePhase.CONNECTING)
        streamJob = launchConnection(connection, previousJob, "Could not connect to the stream. Retry watching.")
    }

    actual fun stopStream() {
        val old = stream
        stream = null
        old?.stopIO()
        streamJob?.cancel()
        streamJob = null
        mutableStreamStatus.value = VoiceConnectionStatus()
        mutableStreamFrame.value = null
    }

    private fun launchConnection(connection: Connection, previousJob: Job?, fallback: String) = scope.launch {
        try {
            previousJob?.join()
            connection.run()
            error("Voice server closed the connection")
        } catch (e: CancellationException) {
            throw e
        } catch (e: Throwable) {
            if (connection.isCurrent) {
                val message = if (e is VoiceFailure) e.message else fallback
                connection.statusFlow.value = VoiceConnectionStatus(connection.channelId, VoicePhase.FAILED, message)
            }
        } finally {
            connection.close()
        }
    }

    actual fun setMuted(muted: Boolean, deafened: Boolean) {
        this.muted = muted
        this.deafened = deafened
        active?.let { it.muted = muted; it.deafened = deafened }
        stream?.deafened = deafened
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
        mutableVideo.value = emptyMap()
    }

    actual fun close() {
        stopStream()
        disconnect()
        scope.cancel()
    }

    private class VoiceFailure(message: String) : Exception(message)

    private companion object {
        /** Bandwidth announced to the SFU so it neither throttles what it forwards nor asks senders to scale down. */
        const val RECEIVE_BITRATE = 20_000_000L
    }

    private class QueuedFrame(val codec: VideoCodec, val data: ByteArray, val arrivedAt: Long, val queuedAt: Long)

    /** Where a video frame's time goes between its first packet and the decoded picture. */
    private class VideoStats {
        private var frames = 0L
        private var waitNanos = 0L
        private var queueNanos = 0L
        private var decodeNanos = 0L
        var dropped = 0L
        var keyframeRequests = 0L

        @Synchronized fun record(arrivedAt: Long, queuedAt: Long, started: Long, done: Long) {
            frames++
            waitNanos += queuedAt - arrivedAt
            queueNanos += started - queuedAt
            decodeNanos += done - started
        }

        @Synchronized fun take(): String {
            fun average(total: Long) = if (frames == 0L) 0 else total / frames / 1_000_000
            val result = "wait=${average(waitNanos)}ms queue=${average(queueNanos)}ms decode=${average(decodeNanos)}ms drop=$dropped pli=$keyframeRequests"
            frames = 0; waitNanos = 0; queueNanos = 0; decodeNanos = 0; dropped = 0; keyframeRequests = 0
            return result
        }
    }

    /** Receive state for one remote video SSRC: reassembly, keyframe gating and a decode worker. */
    private inner class VideoFeed(val connection: Connection, val user: String, val ssrc: Int, scope: CoroutineScope) {
        val buffer = VideoJitterBuffer()
        @Volatile var waitingForKeyframe = true
        var lastPictureLoss = 0L
        val frames = Channel<QueuedFrame>(4)
        private val worker = scope.launch(Dispatchers.Default) {
            var decoder: VideoDecoder? = null
            try {
                for (queued in frames) {
                    val codec = queued.codec
                    try {
                        if (decoder?.codec != codec) {
                            decoder?.close()
                            decoder = null
                            decoder = createVideoDecoder(codec)
                        }
                        val started = System.nanoTime()
                        val image = decoder.decode(queued.data)
                        val done = System.nanoTime()
                        connection.stats.record(queued.arrivedAt, queued.queuedAt, started, done)
                        image?.let { connection.publish(user, it) }
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        Logging.w("Voice", "Video decode failed user=$user codec=$codec: ${e.message}")
                        decoder?.close()
                        decoder = null
                        waitingForKeyframe = true
                    }
                }
            } finally {
                decoder?.close()
            }
        }

        fun close() {
            frames.close()
            worker.cancel()
        }
    }

    private inner class Connection(
        val endpoint: String, val serverId: String, val channelId: String, val daveGroup: String,
        val userId: String, val sessionId: String, val token: String, val isStream: Boolean
    ) {
        val isCurrent get() = (if (isStream) stream else active) === this
        val statusFlow get() = if (isStream) mutableStreamStatus else mutableStatus
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
        @Volatile private var acknowledged = true
        @Volatile private var missedAcks = 0
        @Volatile private var lastRtpAt = 0L
        @Volatile private var secure = false
        @Volatile private var gatewayReady = false
        private var verificationCode = ""
        private val users = mutableSetOf(userId)
        private val sources = ConcurrentHashMap<Int, String>()
        private val videoSources = HashMap<Int, String>()
        private val rtxSources = HashMap<Int, Int>()
        private val videoFeeds = HashMap<Int, VideoFeed>()
        private var mediaScope: CoroutineScope? = null
        private val videoFrames = AtomicLong()
        val stats = VideoStats()
        private var lastEstimate = 0L
        private val playout = mutableMapOf<String, VoicePlayout>()
        private val speaking = ConcurrentHashMap<String, Long>()
        private val rxPackets = AtomicLong()
        private val audioPackets = AtomicLong()
        private val unknownSsrc = AtomicLong()
        private val mixedFrames = AtomicLong()
        private var loggedDecryptFailures = 0L
        private var loggedDecryptSuccesses = 0L
        private var mediaStarted = false
        private var resetCount = 0
        private var pingSequence = 0

        private fun update(phase: VoicePhase = if (!gatewayReady) VoicePhase.CONNECTING else if (secure) VoicePhase.SECURE else VoicePhase.CONNECTED) {
            if (isCurrent) statusFlow.value = VoiceConnectionStatus(channelId, phase, verificationCode = verificationCode, speaking = speaking.keys.toSet())
        }

        suspend fun run() = coroutineScope {
            require(serverId.toULongOrNull() != null && daveGroup.toULongOrNull() != null && userId.toULongOrNull() != null)
            require(endpoint.matches(Regex("[A-Za-z0-9.-]+(?::[0-9]{1,5})?")))
            synchronized(lock) { handle = NativeVoice.create(userId, daveGroup); check(handle != 0L) }
            Logging.i("Voice", "Gateway opened channel=$channelId")
            try {
            val handshake = launch {
                delay(30_000)
                if (!mediaStarted) throw VoiceFailure("Voice handshake timed out. Retry the call.")
            }
            val mediaScope = this
            this@Connection.mediaScope = this
            var attempts = 0
            while (isActive) {
                val resumeTimeout = if (mediaStarted) launch {
                    delay(15_000)
                    if (!gatewayReady) throw VoiceFailure("Voice reconnection timed out. Rejoin the call.")
                } else null
                try {
                    client.webSocket(
                        urlString = "wss://${endpoint.removeSuffix(":80")}/?v=8",
                        request = {
                            timeout {
                                socketTimeoutMillis = HttpTimeoutConfig.INFINITE_TIMEOUT_MS
                            }
                        }
                    ) {
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
                                                val interval = data.getValue("heartbeat_interval").jsonPrimitive.double.toLong().coerceIn(1000, 5000)
                                                acknowledged = true
                                                missedAcks = 0
                                                heartbeat?.cancel()
                                                send(if (mediaStarted) 7 else 0, buildJsonObject {
                                                    put("server_id", serverId); put("user_id", userId)
                                                    put("session_id", sessionId); put("token", token)
                                                    if (mediaStarted) put("seq_ack", sequence) else {
                                                        put("video", true); put("max_dave_protocol_version", 1)
                                                        putJsonArray("streams") {
                                                            add(buildJsonObject {
                                                                put("type", if (isStream) "screen" else "video")
                                                                put("rid", "100"); put("quality", 100)
                                                            })
                                                        }
                                                    }
                                                })
                                                heartbeat = launch {
                                                    delay(interval)
                                                    while (isActive) {
                                                        if (!acknowledged) {
                                                            if (++missedAcks >= 2) throw VoiceFailure("Voice heartbeat timed out")
                                                        } else {
                                                            missedAcks = 0
                                                        }
                                                        acknowledged = false
                                                        send(3, buildJsonObject { put("t", System.currentTimeMillis()); put("seq_ack", sequence) })
                                                        delay(interval)
                                                    }
                                                }
                                            }
                                            2 -> ready(data)
                                            3 -> send(3, buildJsonObject { put("t", System.currentTimeMillis()); put("seq_ack", sequence) })
                                            4 -> {
                                                if (mediaStarted) throw VoiceFailure("Unexpected voice session replacement")
                                                requireE2EE(data["dave_protocol_version"]?.jsonPrimitive?.intOrNull ?: 0)
                                                if (data["mode"]?.jsonPrimitive?.content != mode) throw VoiceFailure("Voice encryption mode mismatch")
                                                val key = data.getValue("secret_key").jsonArray.map {
                                                    it.jsonPrimitive.int.also { value -> require(value in 0..255) }.toByte()
                                                }.toByteArray()
                                                synchronized(lock) { packets = VoicePackets(key, mode == AES_MODE, ssrc, setOf(OPUS_PAYLOAD_TYPE) + VideoCodec.payloadTypes) }
                                                Logging.i("Voice", "Session description mode=$mode dave=${data["dave_protocol_version"]?.jsonPrimitive?.intOrNull} video=${data["video_codec"]?.jsonPrimitive?.contentOrNull}")
                                                initializeDave()
                                                sendSpeaking(0)
                                                send(12, buildJsonObject {
                                                    put("audio_ssrc", ssrc.toLong() and 0xffffffffL); put("video_ssrc", 0); put("rtx_ssrc", 0)
                                                    putJsonArray("streams") { }
                                                })
                                                send(15, buildJsonObject { put("any", 100) })
                                                // A stream viewer only plays the stream's audio, mixed into the call output.
                                                if (!isStream) audio = createVoiceAudio().also { it.setSpeaker(speaker) }
                                                mediaStarted = true
                                                gatewayReady = true
                                                handshake.cancel()
                                                update()
                                                mediaScope.launch { receive() }
                                                mediaScope.launch { pollVideo() }
                                                if (!isStream) {
                                                    mediaScope.launch { captureAudio() }
                                                    mediaScope.launch { playAudio() }
                                                }
                                                mediaScope.launch { keepAlive() }
                                            }
                                            5 -> {
                                                val user = data.getValue("user_id").jsonPrimitive.content
                                                val source = data.getValue("ssrc").jsonPrimitive.content.toUInt().toInt()
                                                if (sources.size < 1000 || sources.containsKey(source)) sources[source] = user
                                                Logging.i("Voice", "Speaking user=$user ssrc=$source flags=${data["speaking"]?.jsonPrimitive?.intOrNull}")
                                            }
                                            6 -> acknowledged = true
                                            9 -> {
                                                attempts = 0
                                                gatewayReady = true
                                                sendSpeaking(0)
                                                update()
                                            }
                                            11 -> {
                                                val ids = data.getValue("user_ids").jsonArray.map { it.jsonPrimitive.content }
                                                require(users.size + ids.size <= 10000)
                                                users.addAll(ids)
                                            }
                                            12 -> {
                                                val user = data.getValue("user_id").jsonPrimitive.content
                                                data["audio_ssrc"]?.jsonPrimitive?.content?.toUIntOrNull()?.toInt()?.let { source ->
                                                    if (source != 0 && (sources.size < 1000 || sources.containsKey(source))) sources[source] = user
                                                }
                                                video(user, data)
                                                send(15, sinkWants())
                                            }
                                            13 -> {
                                                val user = data.getValue("user_id").jsonPrimitive.content
                                                users.remove(user); speaking.remove(user)
                                                removeVideo(user)
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
                                                Logging.i("Voice", "DAVE prepare transition=$id")
                                                transitionReady(id)
                                            }
                                            22 -> {
                                                val id = data.getValue("transition_id").jsonPrimitive.int
                                                val applied = synchronized(lock) { NativeVoice.transition(handle, id, false) } != 0
                                                Logging.i("Voice", "DAVE execute transition=$id applied=$applied")
                                                if (applied) markSecure() else recoverDave()
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
                            Logging.w("Voice", "Gateway closed code=${reason?.code?.toInt()} reason=${reason?.message}")
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
                    Logging.w("Voice", "Voice socket error: ${e.message}")
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
            val udp = DatagramSocket().also {
                socket = it
                it.soTimeout = 5000
                // Keyframes of a 1080p stream arrive as bursts of hundreds of packets.
                it.receiveBufferSize = 4 shl 20
            }
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
            Logging.i("Voice", "UDP ready ssrc=$ssrc port=$port")
            val end = (8 until 72).firstOrNull { response[it] == 0.toByte() } ?: throw VoiceFailure("Invalid voice discovery response")
            val address = response.copyOfRange(8, end).toString(Charsets.US_ASCII)
            require(address.matches(Regex("[0-9a-fA-F:.]+")))
            val externalPort = unsignedShort(response, 72).also { require(it != 0) }
            send(1, buildJsonObject {
                put("protocol", "udp")
                putJsonObject("data") { put("address", address); put("port", externalPort); put("mode", mode) }
                putJsonArray("codecs") {
                    add(buildJsonObject { put("name", "opus"); put("type", "audio"); put("priority", 1000); put("payload_type", OPUS_PAYLOAD_TYPE) })
                    VideoCodec.entries.forEach { codec ->
                        add(buildJsonObject {
                            put("name", codec.name); put("type", "video"); put("priority", codec.priority)
                            put("payload_type", codec.payloadType); put("rtx_payload_type", codec.rtxPayloadType)
                            put("encode", false); put("decode", true)
                        })
                    }
                }
            })
        }

        private suspend fun initializeDave() {
            secure = false
            verificationCode = ""
            val keyPackage = synchronized(lock) { NativeVoice.init(handle, 1) }
            Logging.i("Voice", "DAVE key package ${keyPackage.size} bytes")
            binarySend(26, keyPackage)
            if (mediaStarted) update()
        }

        private suspend fun recoverDave() {
            if (++resetCount > 3) throw VoiceFailure("DAVE key exchange failed repeatedly. Rejoin the call.")
            initializeDave()
        }

        private suspend fun binary(bytes: ByteArray) {
            require(bytes.size >= 3)
            sequence = unsignedShort(bytes, 0)
            val op = bytes[2].toInt() and 255
            val body = bytes.copyOfRange(3, bytes.size)
            when (op) {
                25 -> {
                    Logging.i("Voice", "DAVE external sender ${body.size} bytes")
                    synchronized(lock) { NativeVoice.externalSender(handle, body) }
                }
                27 -> {
                    Logging.i("Voice", "DAVE proposals ${body.size} bytes users=${users.size}")
                    val result = synchronized(lock) { NativeVoice.proposals(handle, body, users.toTypedArray()) }
                    Logging.i("Voice", "DAVE commit+welcome ${result?.size ?: 0} bytes")
                    if (result != null) binarySend(28, result)
                }
                29, 30 -> {
                    require(body.size >= 2)
                    val transition = unsignedShort(body, 0)
                    val result = synchronized(lock) { NativeVoice.commit(handle, body.copyOfRange(2, body.size), users.toTypedArray(), transition, op == 30) }
                    Logging.i("Voice", "DAVE ${if (op == 30) "welcome" else "commit"} transition=$transition result=$result")
                    when (result) {
                        0 -> {
                            if (++resetCount > 3) throw VoiceFailure("DAVE key exchange failed repeatedly. Rejoin the call.")
                            send(31, buildJsonObject { put("transition_id", transition) })
                            initializeDave()
                        }
                        2 -> {
                            resetCount = 0
                            transitionReady(transition)
                        }
                    }
                }
            }
        }

        private suspend fun transitionReady(id: Int) {
            if (id == 0) markSecure() else send(23, buildJsonObject { put("transition_id", id) })
        }

        private fun markSecure() {
            Logging.i("Voice", "DAVE ready")
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
            false
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
                        synchronized(lock) { packets?.startsTalkspurt() }
                        transmitting = true
                    }
                    silence = 5
                    val denoise = Settings.shared.noiseCancellation
                    val packet = synchronized(lock) {
                        NativeVoice.encrypt(handle, ssrc, NativeVoice.encode(handle, pcm, denoise))?.let { packets?.encrypt(it) }
                    }
                    packet?.let {
                        socket?.send(DatagramPacket(it, it.size))
                        lastRtpAt = System.nanoTime()
                    }
                    if (pcm.any { kotlin.math.abs(it.toInt()) > 600 }) speaking[userId] = System.nanoTime()
                } else if (transmitting && gatewayReady) {
                    if (silence-- > 0) {
                        val packet = synchronized(lock) { packets?.encrypt(OPUS_SILENCE) }
                        packet?.let {
                            socket?.send(DatagramPacket(it, it.size))
                            lastRtpAt = System.nanoTime()
                        }
                    } else {
                        if (sendSpeaking(0)) transmitting = false
                    }
                }
                pcm.fill(0)
            }
        }

        private suspend fun receive() {
            val buffer = ByteArray(65536)
            while (currentCoroutineContext().isActive) {
                val datagram = DatagramPacket(buffer, buffer.size)
                try { checkNotNull(socket).receive(datagram) } catch (_: SocketTimeoutException) { continue }
                val data = buffer.copyOf(datagram.length)
                rxPackets.incrementAndGet()
                var feedback: List<ByteArray> = emptyList()
                synchronized(lock) {
                    val packet = packets?.decrypt(data) ?: return@synchronized
                    if (packet.payloadType != OPUS_PAYLOAD_TYPE) {
                        feedback = receiveVideo(packet)
                        return@synchronized
                    }
                    audioPackets.incrementAndGet()
                    val user = sources[packet.ssrc] ?: run { unknownSsrc.incrementAndGet(); return@synchronized }
                    if (user == userId || deafened) return@synchronized
                    if (packet.frame.contentEquals(OPUS_SILENCE)) return@synchronized
                    val opus = NativeVoice.decrypt(handle, user, packet.frame, false) ?: return@synchronized
                    val now = System.nanoTime()
                    playout.getOrPut(user) { VoicePlayout() }.offer(packet.sequence, opus, now)
                    speaking[user] = now
                }
                feedback.forEach { socket?.send(DatagramPacket(it, it.size)) }
            }
        }

        private fun video(user: String, data: JsonObject) {
            fun ssrc(value: JsonElement?) = value?.jsonPrimitive?.content?.toUIntOrNull()?.toInt() ?: 0
            val video = ssrc(data["video_ssrc"])
            if (video == 0) {
                removeVideo(user)
                return
            }
            synchronized(lock) {
                val streams = data["streams"]?.jsonArray.orEmpty().map { it.jsonObject }
                    .filter { it["active"]?.jsonPrimitive?.booleanOrNull != false }
                val media = (listOf(video) + streams.map { ssrc(it["ssrc"]) }).filter { it != 0 }.toSet()
                val rtx = buildMap {
                    ssrc(data["rtx_ssrc"]).takeIf { it != 0 }?.let { put(it, video) }
                    streams.forEach { stream -> ssrc(stream["rtx_ssrc"]).takeIf { it != 0 }?.let { put(it, ssrc(stream["ssrc"])) } }
                }
                videoSources.entries.removeIf { it.value == user && it.key !in media }
                if (videoSources.size + media.size > 1000) return
                media.forEach { videoSources[it] = user }
                rtx.forEach { (repair, source) -> rtxSources[repair] = source }
                Logging.i("Voice", "Video user=$user ssrcs=${media.map { it.toUInt() }}")
            }
        }

        /** Asks for every known video source at full quality. */
        private fun sinkWants() = buildJsonObject {
            put("any", 100)
            synchronized(lock) { videoSources.keys.toList() }.forEach { put((it.toLong() and 0xffffffffL).toString(), 100) }
        }

        private fun removeVideo(user: String) {
            synchronized(lock) {
                val removed = videoSources.filterValues { it == user }.keys
                videoSources.keys.removeAll(removed)
                rtxSources.values.removeAll(removed)
                removed.forEach { videoFeeds.remove(it)?.close() }
            }
            if (isCurrent) {
                if (isStream) mutableStreamFrame.value = null else mutableVideo.update { it - user }
            }
        }

        fun publish(user: String, image: ImageBitmap) {
            if (!isCurrent || synchronized(lock) { videoSources.values.none { it == user } }) return
            videoFrames.incrementAndGet()
            if (isStream) mutableStreamFrame.value = image else mutableVideo.update { it + (user to image) }
        }

        /** Called under [lock]. Returns RTCP feedback to send once the lock is released. */
        private fun receiveVideo(packet: VoicePacket): List<ByteArray> {
            val repair = VideoCodec.forRtx(packet.payloadType)
            val codec = repair ?: VideoCodec.forPayload(packet.payloadType) ?: return emptyList()
            val source = if (repair != null) rtxSources[packet.ssrc] ?: return emptyList() else packet.ssrc
            val user = videoSources[source] ?: run { unknownSsrc.incrementAndGet(); return emptyList() }
            if (user == userId || repair != null && packet.frame.size <= 2) return emptyList()
            val scope = mediaScope ?: return emptyList()
            val feed = videoFeeds[source]?.takeIf { it.user == user } ?: run {
                if (videoFeeds.size >= 64) return emptyList()
                videoFeeds.remove(source)?.close()
                VideoFeed(this, user, source, scope).also { videoFeeds[source] = it }
            }
            if (repair != null) {
                feed.buffer.insert(codec, unsignedShort(packet.frame, 0), packet.timestamp, packet.marker, packet.frame.copyOfRange(2, packet.frame.size), System.nanoTime())
            } else {
                feed.buffer.insert(codec, packet.sequence, packet.timestamp, packet.marker, packet.frame, System.nanoTime())
            }
            return drain(feed, System.nanoTime())
        }

        /** Called under [lock]: turns reassembled frames into decoder input and keyframe/NACK requests. */
        private fun drain(feed: VideoFeed, now: Long): List<ByteArray> {
            val transport = packets ?: return emptyList()
            val poll = feed.buffer.poll(now)
            if (poll.lost) feed.waitingForKeyframe = true
            for (frame in poll.frames) {
                val plain = depacketize(frame.codec, frame.packets)?.let { encrypted ->
                    runCatching { NativeVoice.decrypt(handle, feed.user, encrypted, true) }.getOrNull()
                }
                if (plain == null) {
                    feed.waitingForKeyframe = true
                    continue
                }
                if (feed.waitingForKeyframe) {
                    if (!isKeyframe(frame.codec, plain)) continue
                    feed.waitingForKeyframe = false
                }
                if (!feed.frames.trySend(QueuedFrame(frame.codec, plain, frame.arrivedAt, now)).isSuccess) {
                    feed.waitingForKeyframe = true
                    synchronized(stats) { stats.dropped++ }
                }
            }
            val feedback = mutableListOf<ByteArray>()
            transport.nack(feed.ssrc, poll.nacks)?.let { feedback += it }
            if (feed.waitingForKeyframe && now - feed.lastPictureLoss > 1_000_000_000L) {
                feed.lastPictureLoss = now
                synchronized(stats) { stats.keyframeRequests++ }
                feedback += transport.pictureLoss(feed.ssrc)
            }
            return feedback
        }

        private suspend fun pollVideo() {
            while (currentCoroutineContext().isActive) {
                val feedback = synchronized(lock) {
                    val now = System.nanoTime()
                    val feedback = videoFeeds.values.flatMap { drain(it, now) }.toMutableList()
                    if (videoFeeds.isNotEmpty() && now - lastEstimate >= 500_000_000L) {
                        lastEstimate = now
                        packets?.estimatedBitrate(RECEIVE_BITRATE, videoFeeds.keys.toList())?.let { feedback += it }
                    }
                    feedback
                }
                feedback.forEach { socket?.send(DatagramPacket(it, it.size)) }
                delay(20)
            }
        }

        /** Mixes this (stream) connection's received audio into another connection's output. */
        fun mixInto(mixed: IntArray, now: Long) {
            synchronized(lock) {
                if (handle == 0L) return
                if (!deafened) playout.forEach { (user, buffer) -> buffer.mix(mixed, now) { NativeVoice.decode(handle, user, it) } }
                else playout.clear()
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
                stream?.mixInto(mixed, System.nanoTime())
                var heard = false
                for (i in pcm.indices) {
                    if (mixed[i] != 0) heard = true
                    pcm[i] = mixed[i].coerceIn(-32768, 32767).toShort()
                }
                if (heard) mixedFrames.incrementAndGet()
                checkNotNull(audio).write(pcm)
            }
        }

        private suspend fun keepAlive() {
            var tick = 0
            while (currentCoroutineContext().isActive) {
                val now = System.nanoTime()
                val periodic = tick++ % 25 == 0
                if (periodic) {
                    val ping = ByteBuffer.allocate(8)
                        .putInt(0x1337CAFE)
                        .putInt(pingSequence++)
                        .array()
                    socket?.send(DatagramPacket(ping, ping.size))
                    val report = synchronized(lock) { packets?.senderReport() }
                    report?.let { socket?.send(DatagramPacket(it, it.size)) }
                    val failures = NativeVoice.decryptFailures()
                    val failureDelta = failures - loggedDecryptFailures
                    loggedDecryptFailures = failures
                    val successes = NativeVoice.decryptSuccesses()
                    val successDelta = successes - loggedDecryptSuccesses
                    loggedDecryptSuccesses = successes
                    val packetStats = synchronized(lock) { packets?.takeStats() }
                    Logging.i(
                        "Voice",
                        "udp rx=${rxPackets.getAndSet(0)} audio=${audioPackets.getAndSet(0)} " +
                            "rtpInvalid=${packetStats?.invalidRtp ?: 0} nonAudio=${packetStats?.nonAudio ?: 0} " +
                            "malformed=${packetStats?.malformed ?: 0} transportAuthFail=${packetStats?.authFailed ?: 0} " +
                            "nonceRepeats=${packetStats?.nonceRepeats ?: 0} unknownSsrc=${unknownSsrc.getAndSet(0)} " +
                            "mixed=${mixedFrames.getAndSet(0)} daveOk=+$successDelta daveFail=+$failureDelta " +
                            "playout=${playout.size} sources=${sources.size} video=${videoFeeds.size}/${videoFrames.getAndSet(0)} ${stats.take()}" +
                            if (isStream) " stream" else ""
                    )
                }
                if (packets != null && now - lastRtpAt >= 5_000_000_000L) {
                    val packet = synchronized(lock) { packets?.encrypt(OPUS_SILENCE) }
                    if (packet != null) {
                        try {
                            socket?.send(DatagramPacket(packet, packet.size))
                            lastRtpAt = now
                        } catch (_: Exception) {
                            currentCoroutineContext().ensureActive()
                        }
                    }
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
            socket = null
            audio?.close()
            audio = null
        }

        fun close() {
            stopIO()
            synchronized(lock) {
                if (handle != 0L) NativeVoice.destroy(handle)
                handle = 0L
                packets?.close(); packets = null
                playout.clear()
                videoFeeds.values.forEach { it.close() }
                videoFeeds.clear(); videoSources.clear(); rtxSources.clear()
            }
        }
    }
}
