package me.lampu.lampcord.shared.playback.ffmpeg

import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit

class AudioPlayer(
    private val onPlaybackCompleted: () -> Unit = {},
) {
    private var decoder: AudioDecoder? = null
    private var renderer: AudioRenderer? = null

    private var decodeThread: Thread? = null
    private var renderThread: Thread? = null

    @Volatile
    private var isRunning = false

    @Volatile
    private var isPaused = false

    @Volatile
    private var isSeeking = false

    private val frameQueue = LinkedBlockingQueue<ByteArray>(15) // ~300ms frame buffer
    private val eofSentinel = ByteArray(0)

    private var currentVolume = 1.0f

    // prepareFile runs on an IO thread while close() comes from the UI when the attachment is disposed.
    // A close that lands mid-prepare must not let the finished decoder/renderer be stored, or they leak.
    private val lifecycleLock = Any()
    private var generation = 0L

    fun loadFile(
        path: String,
        headers: Map<String, String> = emptyMap(),
    ): Boolean = prepareFile(path, headers, startPaused = false).also { prepared -> if (prepared) play() }

    // Initializes the decoder and renderer without starting worker threads when paused. This is used for restoring a paused session without producing any audio first.
    fun prepareFile(
        path: String,
        headers: Map<String, String> = emptyMap(),
        startPaused: Boolean = true,
    ): Boolean {
        close()
        val expectedGeneration = synchronized(lifecycleLock) { generation }
        var candidateDecoder: AudioDecoder? = null
        var candidateRenderer: AudioRenderer? = null
        return try {
            val d = AudioDecoder(path, headers).also { candidateDecoder = it }
            val r = AudioRenderer(d.audioFormat).also { candidateRenderer = it }
            r.setVolume(currentVolume)
            synchronized(lifecycleLock) {
                if (generation != expectedGeneration) {
                    runCatching { r.close() }
                    runCatching { d.close() }
                    return false
                }
                decoder = d
                renderer = r
            }
            isPaused = startPaused
            isRunning = false
            true
        } catch (_: Exception) {
            runCatching { candidateRenderer?.close() }
            runCatching { candidateDecoder?.close() }
            isRunning = false
            false
        }
    }

    fun play() {
        if (decodeThread?.isAlive == true || renderThread?.isAlive == true) return

        val d = decoder ?: return
        val r = renderer ?: return

        isRunning = true
        isPaused = false
        frameQueue.clear()

        decodeThread =
            Thread {
                try {
                    val reachedEnd =
                        playDecodedFrames(
                            isRunning = { isRunning },
                            readFrame = {
                                while (isRunning && (isSeeking || isPaused)) {
                                    Thread.sleep(10)
                                }
                                if (isRunning) d.readPCMFrame() else null
                            },
                            writeFrame = { frame ->
                                var pushed = false
                                while (isRunning && !isSeeking && !isPaused && !pushed) {
                                    pushed = frameQueue.offer(frame, 50, TimeUnit.MILLISECONDS)
                                }
                            },
                        )
                    if (reachedEnd) {
                        while (isRunning && !frameQueue.offer(eofSentinel, 50, TimeUnit.MILLISECONDS)) {
                            // Wait until the renderer has room for the completion marker.
                        }
                    }
                } catch (_: InterruptedException) {
                } catch (e: Exception) {
                    e.printStackTrace()
                }
            }.apply {
                priority = Thread.MAX_PRIORITY
                name = "AudioDecoder-Thread"
                start()
            }

        renderThread =
            Thread {
                var reachedEnd = false
                var emptyPollCount = 0
                try {
                    while (isRunning) {
                        if (isPaused) {
                            emptyPollCount = 0
                            Thread.sleep(10)
                            continue
                        }

                        val frame = frameQueue.poll(50, TimeUnit.MILLISECONDS)
                        if (frame == null) {
                            emptyPollCount++
                            if (emptyPollCount >= 100) {
                                reachedEnd = true
                                break
                            }
                            continue
                        }
                        emptyPollCount = 0

                        if (frame === eofSentinel) {
                            reachedEnd = true
                            break
                        }

                        if (frame.isNotEmpty()) {
                            r.writePcmData(frame)
                        }
                    }
                } catch (_: InterruptedException) {
                } catch (e: Exception) {
                    e.printStackTrace()
                } finally {
                    isRunning = false
                    if (reachedEnd) {
                        onPlaybackCompleted()
                    }
                }
            }.apply {
                priority = Thread.MAX_PRIORITY
                name = "AudioRenderer-Thread"
                start()
            }
    }

    fun pause() {
        isPaused = true
        renderer?.pause()
    }

    fun resume() {
        if (decodeThread?.isAlive != true && renderThread?.isAlive != true) {
            play()
        } else {
            renderer?.resume()
            isPaused = false
        }
    }

    fun seekTo(position: Long) {
        val d = decoder ?: return
        val dur = getDuration()
        if (dur == 0L || position <= dur) {
            isSeeking = true
            try {
                frameQueue.clear()
                renderer?.flush()
                d.seekTo(position)
            } catch (e: Exception) {
                e.printStackTrace()
            } finally {
                isSeeking = false
            }
        }
    }

    fun setVolume(volume: Float) {
        currentVolume = volume
        renderer?.setVolume(volume)
    }

    fun getVolume(): Float = renderer?.getVolume() ?: 1.0f

    fun getDuration(): Long = decoder?.duration ?: 0L

    fun getCurrentPosition(): Long {
        val rawPos = decoder?.getCurrentPosition() ?: 0L
        val bufferedMs = frameQueue.size * 20L
        return (rawPos - bufferedMs).coerceAtLeast(0L)
    }

    fun isPlaying(): Boolean = isRunning && !isPaused && (renderer?.isPlaying() ?: false)

    internal val isPrepared: Boolean
        get() = decoder != null && renderer != null

    internal val workersActive: Boolean
        get() = decodeThread?.isAlive == true || renderThread?.isAlive == true

    fun close() {
        isRunning = false
        isPaused = false
        isSeeking = false

        decodeThread?.interrupt()
        renderThread?.interrupt()

        frameQueue.clear()

        try {
            decodeThread?.join(300)
            renderThread?.join(300)
        } catch (e: InterruptedException) {
            Thread.currentThread().interrupt()
        }

        val r: AudioRenderer?
        val d: AudioDecoder?
        synchronized(lifecycleLock) {
            generation++
            r = renderer
            d = decoder
            renderer = null
            decoder = null
        }
        r?.close()
        d?.close()

        decodeThread = null
        renderThread = null
    }
}

internal inline fun playDecodedFrames(
    isRunning: () -> Boolean,
    readFrame: () -> ByteArray?,
    writeFrame: (ByteArray) -> Unit,
): Boolean {
    while (isRunning()) {
        val data = readFrame() ?: return isRunning()
        writeFrame(data)
    }
    return false
}
