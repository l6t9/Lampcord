package me.lampu.lampcord.shared.playback.ffmpeg

import java.beans.PropertyEditorSupport
import java.io.Closeable
import java.io.File
import java.io.IOException
import java.nio.Buffer
import java.nio.charset.Charset
import java.util.*
import java.util.concurrent.*

abstract class FrameGrabber : Closeable {
    companion object {
        @JvmStatic
        val list =
            LinkedList(
                listOf(
                    "DC1394",
                    "FlyCapture",
                    "FlyCapture2",
                    "OpenKinect",
                    "OpenKinect2",
                    "RealSense",
                    "RealSense2",
                    "PS3Eye",
                    "VideoInput",
                    "OpenCV",
                    "FFmpeg",
                    "IPCamera",
                ),
            )

        @JvmStatic
        fun init() {
            for (name in list) {
                try {
                    val c = get(name)
                    c.getMethod("tryLoad").invoke(null)
                } catch (t: Throwable) {
                    continue
                }
            }
        }

        @JvmStatic
        fun getDefault(): Class<out FrameGrabber>? {
            for (name in list) {
                try {
                    val c = get(name)
                    c.getMethod("tryLoad").invoke(null)
                    var mayContainCameras = false
                    try {
                        @Suppress("UNCHECKED_CAST")
                        val s = c.getMethod("getDeviceDescriptions").invoke(null) as kotlin.Array<String>
                        if (s.isNotEmpty()) {
                            mayContainCameras = true
                        }
                    } catch (t: Throwable) {
                        if (t.cause is UnsupportedOperationException) {
                            mayContainCameras = true
                        }
                    }
                    if (mayContainCameras) {
                        return c
                    }
                } catch (t: Throwable) {
                    continue
                }
            }
            return null
        }

        @JvmStatic
        @Throws(Exception::class)
        fun get(className: String): Class<out FrameGrabber> {
            val fullClassName = FrameGrabber::class.java.`package`.name + "." + className
            return try {
                Class.forName(fullClassName).asSubclass(FrameGrabber::class.java)
            } catch (e: ClassNotFoundException) {
                val className2 = fullClassName + "FrameGrabber"
                try {
                    Class.forName(className2).asSubclass(FrameGrabber::class.java)
                } catch (ex: ClassNotFoundException) {
                    throw Exception("Could not get FrameGrabber class for $fullClassName or $className2", e)
                }
            }
        }

        @JvmStatic
        @Throws(Exception::class)
        fun create(
            c: Class<out FrameGrabber>,
            p: Class<*>,
            o: Any,
        ): FrameGrabber =
            try {
                c.getConstructor(p).newInstance(o)
            } catch (ex: java.lang.Exception) {
                throw Exception("Could not create new ${c.simpleName}($o)", ex.cause ?: ex)
            }

        @JvmStatic
        @Throws(Exception::class)
        fun createDefault(deviceFile: File): FrameGrabber = create(getDefault()!!, File::class.java, deviceFile)

        @JvmStatic
        @Throws(Exception::class)
        fun createDefault(devicePath: String): FrameGrabber = create(getDefault()!!, String::class.java, devicePath)

        @JvmStatic
        @Throws(Exception::class)
        fun createDefault(deviceNumber: Int): FrameGrabber =
            try {
                create(getDefault()!!, Int::class.javaPrimitiveType!!, deviceNumber)
            } catch (ex: java.lang.Exception) {
                create(getDefault()!!, Int::class.javaObjectType, deviceNumber)
            }

        const val SENSOR_PATTERN_RGGB = 0L
        const val SENSOR_PATTERN_GBRG = 1L shl 32
        const val SENSOR_PATTERN_GRBG = 1L
        const val SENSOR_PATTERN_BGGR = (1L shl 32) or 1L
    }

    class PropertyEditor : PropertyEditorSupport() {
        override fun getAsText(): String {
            val c = value as? Class<*> ?: return "null"
            return c.simpleName
                .split("FrameGrabber".toRegex())
                .dropLastWhile { it.isEmpty() }
                .toTypedArray()[0]
        }

        override fun setAsText(s: String?) {
            if (s == null) {
                value = null
                return
            }
            try {
                value = get(s)
            } catch (ex: java.lang.Exception) {
                throw IllegalArgumentException(ex)
            }
        }

        override fun getTags(): kotlin.Array<String> = list.toTypedArray()
    }

    enum class ImageMode {
        COLOR,
        GRAY,
        RAW,
    }

    enum class SampleMode {
        SHORT,
        FLOAT,
        RAW,
    }

    open var videoStream = -1
    open var audioStream = -1
    open var videoDisposition = 0
    open var audioDisposition = 0
    open var format: String? = null
    open var videoCodecName: String? = null
    open var audioCodecName: String? = null
    open var imageWidth = 0
    open var imageHeight = 0
    open var audioChannels = 0
    open var imageMode = ImageMode.COLOR
    open var sensorPattern = -1L
    open var pixelFormat = -1
    open var videoCodec = 0
    open var videoBitrate = 0
    open var imageScalingFlags = 0
    open var aspectRatio = 0.0
    open var frameRate = 0.0
    open var sampleMode = SampleMode.SHORT
    open var sampleFormat = -1
    open var audioCodec = 0
    open var audioBitrate = 0
    open var sampleRate = 0
    open var isTriggerMode = false
    open var bitsPerPixel = 0
    open var timeout = 10000
    open var numBuffers = 4
    open var gamma = 0.0
    open var isDeinterlace = false
    open var charset: Charset = Charset.defaultCharset()
    open var options: MutableMap<String, String> = HashMap()
    open var videoOptions: MutableMap<String, String> = HashMap()
    open var audioOptions: MutableMap<String, String> = HashMap()
    open var metadata: MutableMap<String, String> = HashMap()
    open var videoMetadata: MutableMap<String, String> = HashMap()
    open var audioMetadata: MutableMap<String, String> = HashMap()
    open var videoSideData: MutableMap<String, Buffer> = HashMap()
    open var audioSideData: MutableMap<String, Buffer> = HashMap()
    open var frameNumber = 0
    open var timestamp = 0L
    open var maxDelay = -1
    open var startTime = 0L

    open fun getLengthInFrames(): Int = 0

    open fun getLengthInTime(): Long = 0

    abstract fun hasVideo(): Boolean

    abstract fun hasAudio(): Boolean

    open class Exception : IOException {
        constructor(message: String) : super(message)
        constructor(message: String, cause: Throwable) : super(message, cause)
    }

    @Throws(Exception::class)
    abstract fun start()

    @Throws(Exception::class)
    abstract fun stop()

    @Throws(Exception::class)
    abstract fun trigger()

    @Throws(Exception::class)
    override fun close() {
        stop()
        release()
    }

    @Throws(Exception::class)
    abstract fun grab(): Frame?

    @Throws(Exception::class)
    open fun grabFrame(): Frame? = grab()

    @Throws(Exception::class)
    abstract fun release()

    @Throws(Exception::class)
    open fun restart() {
        stop()
        start()
    }

    @Throws(Exception::class)
    open fun flush() {
        for (i in 0 until numBuffers + 1) {
            grab()
        }
    }

    private val executor = Executors.newSingleThreadExecutor()
    private var future: Future<Void>? = null
    private var delayedFrame: Frame? = null
    private var delayedTime = 0L

    open fun delayedGrab(delayTime: Long) {
        delayedFrame = null
        delayedTime = 0
        val start = System.nanoTime() / 1000
        if (future != null && !future!!.isDone) {
            return
        }
        future =
            executor.submit(
                Callable<Void> {
                    do {
                        delayedFrame = grab()
                        delayedTime = System.nanoTime() / 1000 - start
                    } while (delayedTime < delayTime)
                    null
                },
            )
    }

    @Throws(InterruptedException::class, ExecutionException::class)
    open fun getDelayedTime(): Long {
        if (future == null) return 0
        future!!.get()
        return delayedTime
    }

    @Throws(InterruptedException::class, ExecutionException::class)
    open fun getDelayedFrame(): Frame? {
        if (future == null) return null
        future!!.get()
        return delayedFrame
    }

    open class Array(
        frameGrabbers: kotlin.Array<FrameGrabber>,
    ) {
        private var grabbedFrames: kotlin.Array<Frame?>? = null
        private var latencies: LongArray? = null
        private var bestLatencies: LongArray? = null
        private var lastNewestTimestamp = 0L
        private var bestInterval = Long.MAX_VALUE

        var frameGrabbers: kotlin.Array<FrameGrabber>? = null
            set(value) {
                field = value
                grabbedFrames = arrayOfNulls(value?.size ?: 0)
                latencies = LongArray(value?.size ?: 0)
                bestLatencies = null
                lastNewestTimestamp = 0
            }

        init {
            this.frameGrabbers = frameGrabbers
        }

        fun size(): Int = frameGrabbers?.size ?: 0

        @Throws(Exception::class)
        fun start() {
            frameGrabbers?.forEach { it.start() }
        }

        @Throws(Exception::class)
        fun stop() {
            frameGrabbers?.forEach { it.stop() }
        }

        @Throws(Exception::class)
        fun trigger() {
            frameGrabbers?.forEach { if (it.isTriggerMode) it.trigger() }
        }

        @Throws(Exception::class)
        fun grab(): kotlin.Array<Frame?>? {
            val grabbers = frameGrabbers ?: return null
            if (grabbers.size == 1) {
                grabbedFrames!![0] = grabbers[0].grab()
                return grabbedFrames
            }

            var newestTimestamp = 0L
            var unsynchronized = false
            for (i in grabbers.indices) {
                grabbedFrames!![i] = grabbers[i].grab()
                if (grabbedFrames!![i] != null) {
                    newestTimestamp = Math.max(newestTimestamp, grabbers[i].timestamp)
                }
                if (grabbers[i].javaClass != grabbers[(i + 1) % grabbers.size].javaClass) {
                    unsynchronized = true
                }
            }
            if (unsynchronized) return grabbedFrames

            for (i in grabbers.indices) {
                if (grabbedFrames!![i] != null) {
                    latencies!![i] = newestTimestamp - Math.max(0, grabbers[i].timestamp)
                }
            }
            if (bestLatencies == null) {
                bestLatencies = latencies!!.copyOf()
            } else {
                var sum1 = 0L
                var sum2 = 0L
                for (i in grabbers.indices) {
                    sum1 += latencies!![i]
                    sum2 += bestLatencies!![i]
                }
                if (sum1 < sum2) {
                    bestLatencies = latencies!!.copyOf()
                }
            }

            bestInterval = Math.min(bestInterval, newestTimestamp - lastNewestTimestamp)
            for (i in bestLatencies!!.indices) {
                bestLatencies!![i] = Math.min(bestLatencies!![i], bestInterval * 9 / 10)
            }

            for (j in 0..1) {
                for (i in grabbers.indices) {
                    if (grabbers[i].isTriggerMode || grabbedFrames!![i] == null) continue
                    var latency = (newestTimestamp - Math.max(0, grabbers[i].timestamp)).toInt()
                    while (latency - bestLatencies!![i] > 0.1 * bestLatencies!![i]) {
                        grabbedFrames!![i] = grabbers[i].grab()
                        if (grabbedFrames!![i] == null) break
                        latency = (newestTimestamp - Math.max(0, grabbers[i].timestamp)).toInt()
                        if (latency < 0) {
                            newestTimestamp = Math.max(0, grabbers[i].timestamp)
                            break
                        }
                    }
                }
            }
            lastNewestTimestamp = newestTimestamp
            return grabbedFrames
        }

        @Throws(Exception::class)
        fun release() {
            frameGrabbers?.forEach { it.release() }
        }
    }

    open fun createArray(frameGrabbers: kotlin.Array<FrameGrabber>): Array = Array(frameGrabbers)

    @Throws(Exception::class, InterruptedException::class)
    open fun grabAtFrameRate(): Frame? {
        val frame = grab()
        if (frame != null) {
            waitForTimestamp(frame)
        }
        return frame
    }

    @Throws(InterruptedException::class)
    open fun waitForTimestamp(frame: Frame): Boolean {
        if (startTime == 0L) {
            startTime = System.nanoTime() / 1000 - frame.timestamp
        } else {
            val delay = frame.timestamp - (System.nanoTime() / 1000 - startTime)
            if (delay > 0) {
                Thread.sleep(delay / 1000, (delay % 1000).toInt() * 1000)
                return true
            }
        }
        return false
    }

    open fun resetStartTime() {
        startTime = 0
    }
}
