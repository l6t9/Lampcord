package me.lampu.lampcord.shared.voice

/** Calls and destruction are serialized by the owning voice connection. */
internal object NativeVoice {
    init {
        if (System.getProperty("java.vm.name").orEmpty().contains("Dalvik", ignoreCase = true)) {
            System.loadLibrary("lampcord_voice")
        } else {
            val name = System.mapLibraryName("lampcord_voice")
            val resource = checkNotNull(javaClass.getResourceAsStream("/voice/$name")) {
                "Voice native library is missing; rebuild the desktop client"
            }
            val file = java.nio.file.Files.createTempFile("lampcord-voice-", name).toFile()
            file.deleteOnExit()
            resource.use { input -> file.outputStream().use { input.copyTo(it) } }
            System.load(file.absolutePath)
        }
    }

    external fun create(user: String, channel: String): Long
    external fun destroy(handle: Long)
    external fun init(handle: Long, version: Int): ByteArray
    external fun externalSender(handle: Long, payload: ByteArray)
    external fun proposals(handle: Long, payload: ByteArray, users: Array<String>): ByteArray?
    external fun commit(handle: Long, payload: ByteArray, users: Array<String>, transition: Int, welcome: Boolean): Int
    external fun transition(handle: Long, id: Int, prepare: Boolean)
    external fun removeUser(handle: Long, user: String)
    external fun authenticator(handle: Long): ByteArray
    external fun encode(handle: Long, pcm: ShortArray): ByteArray
    external fun encrypt(handle: Long, ssrc: Int, opus: ByteArray): ByteArray?
    external fun decrypt(handle: Long, user: String, frame: ByteArray): ByteArray?
    external fun decode(handle: Long, user: String, opus: ByteArray?): ShortArray?
    external fun aead(encrypt: Boolean, aes: Boolean, key: ByteArray, nonce: ByteArray, header: ByteArray, payload: ByteArray): ByteArray?
}

internal interface VoiceAudio : AutoCloseable {
    /** Exactly 20ms of 48kHz stereo PCM. Reads/writes block to pace the media loops. */
    fun read(pcm: ShortArray)
    fun write(pcm: ShortArray)
    fun setSpeaker(enabled: Boolean)
}

internal expect fun createVoiceAudio(): VoiceAudio
