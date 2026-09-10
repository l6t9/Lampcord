package me.lampu.lampcord.shared.voice

import android.Manifest
import android.content.pm.PackageManager
import android.media.*
import android.media.audiofx.AcousticEchoCanceler
import android.media.audiofx.NoiseSuppressor
import android.os.Build
import android.os.Handler
import android.os.Looper
import me.lampu.lampcord.shared.utils.AndroidContextProvider

internal actual fun createVoiceAudio(): VoiceAudio = AndroidVoiceAudio()

@Suppress("DEPRECATION")
private class AndroidVoiceAudio : VoiceAudio {
    private val context = AndroidContextProvider.applicationContext
    private val manager = context.getSystemService(AudioManager::class.java)
    private val attributes = AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_VOICE_COMMUNICATION)
        .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build()
    private val oldMode = manager.mode
    private val oldSpeaker = manager.isSpeakerphoneOn
    @Volatile private var closed = false
    @Volatile private var focused = true
    private var speaker = false
    private var input: AudioRecord? = null
    private var output: AudioTrack? = null
    private var echo: AcousticEchoCanceler? = null
    private var noise: NoiseSuppressor? = null
    private val mono = ShortArray(960)
    private val focusListener = AudioManager.OnAudioFocusChangeListener { focused = it > 0 }
    private val focusRequest = if (Build.VERSION.SDK_INT >= 26) AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
        .setAudioAttributes(attributes).setOnAudioFocusChangeListener(focusListener, Handler(Looper.getMainLooper())).build() else null
    private val devices = object : AudioDeviceCallback() {
        override fun onAudioDevicesAdded(addedDevices: Array<out AudioDeviceInfo>) { if (!closed) setSpeaker(speaker) }
        override fun onAudioDevicesRemoved(removedDevices: Array<out AudioDeviceInfo>) { if (!closed) setSpeaker(speaker) }
    }

    init {
        try {
            check(context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED) { "Microphone permission required" }
            val focus = if (Build.VERSION.SDK_INT >= 26) manager.requestAudioFocus(focusRequest!!)
                else manager.requestAudioFocus(focusListener, AudioManager.STREAM_VOICE_CALL, AudioManager.AUDIOFOCUS_GAIN)
            check(focus == AudioManager.AUDIOFOCUS_REQUEST_GRANTED) { "Audio is in use by another call" }
            manager.mode = AudioManager.MODE_IN_COMMUNICATION
            val inputSize = AudioRecord.getMinBufferSize(48000, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT)
            check(inputSize > 0)
            input = AudioRecord(MediaRecorder.AudioSource.VOICE_COMMUNICATION, 48000, AudioFormat.CHANNEL_IN_MONO,
                AudioFormat.ENCODING_PCM_16BIT, maxOf(inputSize, 1920 * 3))
            check(input!!.state == AudioRecord.STATE_INITIALIZED)
            val session = input!!.audioSessionId
            if (AcousticEchoCanceler.isAvailable()) echo = AcousticEchoCanceler.create(session)?.also { it.enabled = true }
            if (NoiseSuppressor.isAvailable()) noise = NoiseSuppressor.create(session)?.also { it.enabled = true }
            val outputSize = AudioTrack.getMinBufferSize(48000, AudioFormat.CHANNEL_OUT_STEREO, AudioFormat.ENCODING_PCM_16BIT)
            check(outputSize > 0)
            output = AudioTrack.Builder().setAudioAttributes(attributes)
                .setAudioFormat(AudioFormat.Builder().setSampleRate(48000).setEncoding(AudioFormat.ENCODING_PCM_16BIT).setChannelMask(AudioFormat.CHANNEL_OUT_STEREO).build())
                .setTransferMode(AudioTrack.MODE_STREAM).setBufferSizeInBytes(maxOf(outputSize, 3840 * 3)).build()
            check(output!!.state == AudioTrack.STATE_INITIALIZED)
            output!!.play()
            manager.registerAudioDeviceCallback(devices, Handler(Looper.getMainLooper()))
            input!!.startRecording()
            check(input!!.recordingState == AudioRecord.RECORDSTATE_RECORDING)
        } catch (e: Exception) {
            close()
            throw e
        }
    }

    override fun read(pcm: ShortArray) {
        var offset = 0
        while (offset < mono.size && !closed) {
            check(focused) { "Audio focus was lost to another app" }
            val count = checkNotNull(input).read(mono, offset, mono.size - offset, AudioRecord.READ_BLOCKING)
            check(count > 0) { "Microphone stopped" }
            offset += count
        }
        check(!closed)
        for (i in mono.indices) { pcm[i * 2] = mono[i]; pcm[i * 2 + 1] = mono[i] }
        mono.fill(0)
    }

    override fun write(pcm: ShortArray) {
        var offset = 0
        while (offset < pcm.size && !closed) {
            val count = checkNotNull(output).write(pcm, offset, pcm.size - offset, AudioTrack.WRITE_BLOCKING)
            check(count > 0) { "Audio output stopped" }
            offset += count
        }
        check(!closed)
    }

    @Synchronized override fun setSpeaker(enabled: Boolean) {
        if (closed) return
        speaker = enabled
        if (Build.VERSION.SDK_INT >= 31) {
            val devices = manager.availableCommunicationDevices
            val priority = if (enabled) listOf(AudioDeviceInfo.TYPE_BUILTIN_SPEAKER) else listOf(
                AudioDeviceInfo.TYPE_BLE_HEADSET, AudioDeviceInfo.TYPE_BLUETOOTH_SCO,
                AudioDeviceInfo.TYPE_WIRED_HEADSET, AudioDeviceInfo.TYPE_USB_HEADSET,
                AudioDeviceInfo.TYPE_WIRED_HEADPHONES, AudioDeviceInfo.TYPE_BUILTIN_EARPIECE,
                AudioDeviceInfo.TYPE_BUILTIN_SPEAKER
            )
            val bluetoothAllowed = context.checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT) == PackageManager.PERMISSION_GRANTED
            val device = priority.firstNotNullOfOrNull { type ->
                devices.firstOrNull { it.type == type && (bluetoothAllowed || type !in listOf(AudioDeviceInfo.TYPE_BLUETOOTH_SCO, AudioDeviceInfo.TYPE_BLE_HEADSET)) }
            }
            if (device != null) manager.setCommunicationDevice(device)
        } else {
            manager.isSpeakerphoneOn = enabled
        }
    }

    @Synchronized override fun close() {
        if (closed) return
        closed = true
        manager.unregisterAudioDeviceCallback(devices)
        runCatching { input?.stop() }
        runCatching { output?.stop() }
        echo?.release(); noise?.release()
        input?.release(); output?.release()
        if (Build.VERSION.SDK_INT >= 31) manager.clearCommunicationDevice() else manager.isSpeakerphoneOn = oldSpeaker
        manager.mode = oldMode
        if (Build.VERSION.SDK_INT >= 26) manager.abandonAudioFocusRequest(focusRequest!!) else manager.abandonAudioFocus(focusListener)
    }
}
