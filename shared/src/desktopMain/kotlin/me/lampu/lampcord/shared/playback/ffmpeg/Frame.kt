package me.lampu.lampcord.shared.playback.ffmpeg

import org.bytedeco.javacpp.*
import org.bytedeco.javacpp.indexer.*
import java.nio.*
import java.util.EnumSet

class Frame :
    AutoCloseable,
    Indexable,
    Cloneable {
    var keyFrame: Boolean = false
    var pictType: Char = '\u0000'

    companion object {
        const val DEPTH_BYTE = -8
        const val DEPTH_UBYTE = 8
        const val DEPTH_SHORT = -16
        const val DEPTH_USHORT = 16
        const val DEPTH_INT = -32
        const val DEPTH_LONG = -64
        const val DEPTH_FLOAT = 32
        const val DEPTH_DOUBLE = 64

        @JvmStatic
        fun pixelSize(depth: Int): Int = Math.abs(depth) / 8
    }

    enum class Type {
        VIDEO,
        AUDIO,
        DATA,
        SUBTITLE,
        ATTACHMENT,
    }

    var imageWidth: Int = 0
    var imageHeight: Int = 0
    var imageDepth: Int = 0
    var imageChannels: Int = 0
    var imageStride: Int = 0
    var image: kotlin.Array<Buffer?>? = null

    var sampleRate: Int = 0
    var audioChannels: Int = 0
    var samples: kotlin.Array<Buffer?>? = null

    var data: ByteBuffer? = null
    var streamIndex: Int = -1
    var type: Type? = null
    var opaque: Any? = null
    var timestamp: Long = 0

    constructor()

    constructor(width: Int, height: Int, depth: Int, channels: Int) : this(
        width,
        height,
        depth,
        channels,
        ((width * channels * pixelSize(depth) + 7) and 7.inv()) / pixelSize(depth),
    )

    constructor(width: Int, height: Int, depth: Int, channels: Int, imageStride: Int) {
        this.imageWidth = width
        this.imageHeight = height
        this.imageDepth = depth
        this.imageChannels = channels
        this.imageStride = imageStride
        this.pictType = '\u0000'
        this.image = arrayOfNulls<Buffer>(1)
        this.data = null
        this.streamIndex = -1
        this.type = null

        val pointer = BytePointer((imageHeight * imageStride * pixelSize(depth)).toLong())
        val buffer = pointer.asByteBuffer()
        when (imageDepth) {
            DEPTH_BYTE, DEPTH_UBYTE -> image!![0] = buffer
            DEPTH_SHORT, DEPTH_USHORT -> image!![0] = buffer.asShortBuffer()
            DEPTH_INT -> image!![0] = buffer.asIntBuffer()
            DEPTH_LONG -> image!![0] = buffer.asLongBuffer()
            DEPTH_FLOAT -> image!![0] = buffer.asFloatBuffer()
            DEPTH_DOUBLE -> image!![0] = buffer.asDoubleBuffer()
            else -> throw UnsupportedOperationException("Unsupported depth value: $imageDepth")
        }
        opaque = arrayOf<Pointer>(pointer.retainReference<Pointer>())
    }

    override fun <I : Indexer?> createIndexer(direct: Boolean): I? {
        val sizes = longArrayOf(imageHeight.toLong(), imageWidth.toLong(), imageChannels.toLong())
        val strides = longArrayOf(imageStride.toLong(), imageChannels.toLong(), 1L)
        val img = image ?: return null
        val buffer = img[0] ?: return null
        val array = if (buffer.hasArray()) buffer.array() else null

        @Suppress("UNCHECKED_CAST")
        return when (imageDepth) {
            DEPTH_UBYTE -> {
                if (array != null) {
                    UByteIndexer.create(array as ByteArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    UByteIndexer.create(buffer as ByteBuffer, sizes, strides).indexable(this) as I
                } else {
                    UByteIndexer.create(BytePointer(buffer as ByteBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_BYTE -> {
                if (array != null) {
                    ByteIndexer.create(array as ByteArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    ByteIndexer.create(buffer as ByteBuffer, sizes, strides).indexable(this) as I
                } else {
                    ByteIndexer.create(BytePointer(buffer as ByteBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_USHORT -> {
                if (array != null) {
                    UShortIndexer.create(array as ShortArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    UShortIndexer.create(buffer as ShortBuffer, sizes, strides).indexable(this) as I
                } else {
                    UShortIndexer.create(ShortPointer(buffer as ShortBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_SHORT -> {
                if (array != null) {
                    ShortIndexer.create(array as ShortArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    ShortIndexer.create(buffer as ShortBuffer, sizes, strides).indexable(this) as I
                } else {
                    ShortIndexer.create(ShortPointer(buffer as ShortBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_INT -> {
                if (array != null) {
                    IntIndexer.create(array as IntArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    IntIndexer.create(buffer as IntBuffer, sizes, strides).indexable(this) as I
                } else {
                    IntIndexer.create(IntPointer(buffer as IntBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_LONG -> {
                if (array != null) {
                    LongIndexer.create(array as LongArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    LongIndexer.create(buffer as LongBuffer, sizes, strides).indexable(this) as I
                } else {
                    LongIndexer.create(LongPointer(buffer as LongBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_FLOAT -> {
                if (array != null) {
                    FloatIndexer.create(array as FloatArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    FloatIndexer.create(buffer as FloatBuffer, sizes, strides).indexable(this) as I
                } else {
                    FloatIndexer.create(FloatPointer(buffer as FloatBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            DEPTH_DOUBLE -> {
                if (array != null) {
                    DoubleIndexer.create(array as DoubleArray, sizes, strides).indexable(this) as I
                } else if (direct) {
                    DoubleIndexer.create(buffer as DoubleBuffer, sizes, strides).indexable(this) as I
                } else {
                    DoubleIndexer.create(DoublePointer(buffer as DoubleBuffer), sizes, strides, false).indexable(this) as I
                }
            }

            else -> {
                null
            }
        }
    }

    override fun clone(): Frame {
        val newFrame = Frame()
        newFrame.imageWidth = imageWidth
        newFrame.imageHeight = imageHeight
        newFrame.imageDepth = imageDepth
        newFrame.imageChannels = imageChannels
        newFrame.imageStride = imageStride
        newFrame.keyFrame = keyFrame
        newFrame.pictType = pictType
        newFrame.streamIndex = streamIndex
        newFrame.type = type
        val opaque = arrayOfNulls<Pointer>(3)
        newFrame.opaque = opaque

        if (image != null) {
            newFrame.image = arrayOfNulls<Buffer>(image!!.size)
            opaque[0] = cloneBufferArray(image!!, newFrame.image!!)
        }

        newFrame.audioChannels = audioChannels
        newFrame.sampleRate = sampleRate
        if (samples != null) {
            newFrame.samples = arrayOfNulls<Buffer>(samples!!.size)
            opaque[1] = cloneBufferArray(samples!!, newFrame.samples!!)
        }

        if (data != null) {
            val dst = arrayOfNulls<Buffer>(1)
            opaque[2] = cloneBufferArray(arrayOf(data), dst)
            newFrame.data = dst[0] as ByteBuffer?
        }

        newFrame.timestamp = timestamp
        return newFrame
    }

    private fun cloneBufferArray(
        srcBuffers: kotlin.Array<Buffer?>,
        clonedBuffers: kotlin.Array<Buffer?>,
    ): Pointer? {
        var opaque: Pointer? = null
        if (srcBuffers.isNotEmpty()) {
            var totalCapacity = 0
            for (i in srcBuffers.indices) {
                srcBuffers[i]?.rewind()
                totalCapacity += srcBuffers[i]?.capacity() ?: 0
            }

            val first = srcBuffers[0]
            when (first) {
                is ByteBuffer -> {
                    val pointer = BytePointer(totalCapacity.toLong())
                    for (i in srcBuffers.indices) {
                        val limit = pointer.position() + (srcBuffers[i]?.limit() ?: 0)
                        pointer.limit(limit)
                        clonedBuffers[i] = pointer.asBuffer().put(srcBuffers[i] as ByteBuffer)
                        pointer.position(limit)
                    }
                    opaque = pointer
                }

                is ShortBuffer -> {
                    val pointer = ShortPointer(totalCapacity.toLong())
                    for (i in srcBuffers.indices) {
                        val limit = pointer.position() + (srcBuffers[i]?.limit() ?: 0)
                        pointer.limit(limit)
                        clonedBuffers[i] = pointer.asBuffer().put(srcBuffers[i] as ShortBuffer)
                        pointer.position(limit)
                    }
                    opaque = pointer
                }

                is IntBuffer -> {
                    val pointer = IntPointer(totalCapacity.toLong())
                    for (i in srcBuffers.indices) {
                        val limit = pointer.position() + (srcBuffers[i]?.limit() ?: 0)
                        pointer.limit(limit)
                        clonedBuffers[i] = pointer.asBuffer().put(srcBuffers[i] as IntBuffer)
                        pointer.position(limit)
                    }
                    opaque = pointer
                }

                is LongBuffer -> {
                    val pointer = LongPointer(totalCapacity.toLong())
                    for (i in srcBuffers.indices) {
                        val limit = pointer.position() + (srcBuffers[i]?.limit() ?: 0)
                        pointer.limit(limit)
                        clonedBuffers[i] = pointer.asBuffer().put(srcBuffers[i] as LongBuffer)
                        pointer.position(limit)
                    }
                    opaque = pointer
                }

                is FloatBuffer -> {
                    val pointer = FloatPointer(totalCapacity.toLong())
                    for (i in srcBuffers.indices) {
                        val limit = pointer.position() + (srcBuffers[i]?.limit() ?: 0)
                        pointer.limit(limit)
                        clonedBuffers[i] = pointer.asBuffer().put(srcBuffers[i] as FloatBuffer)
                        pointer.position(limit)
                    }
                    opaque = pointer
                }

                is DoubleBuffer -> {
                    val pointer = DoublePointer(totalCapacity.toLong())
                    for (i in srcBuffers.indices) {
                        val limit = pointer.position() + (srcBuffers[i]?.limit() ?: 0)
                        pointer.limit(limit)
                        clonedBuffers[i] = pointer.asBuffer().put(srcBuffers[i] as DoubleBuffer)
                        pointer.position(limit)
                    }
                    opaque = pointer
                }

                else -> {
                    if (first == null) return null
                    throw UnsupportedOperationException("Unsupported buffer type: ${first.javaClass.simpleName}")
                }
            }

            for (i in srcBuffers.indices) {
                srcBuffers[i]?.rewind()
                clonedBuffers[i]?.rewind()
            }
        }
        return opaque?.retainReference<Pointer>()
    }

    fun getTypes(): EnumSet<Type> {
        val type = EnumSet.noneOf(Type::class.java)
        if (image != null) type.add(Type.VIDEO)
        if (samples != null) type.add(Type.AUDIO)
        if (data != null) type.add(Type.DATA)
        return type
    }

    override fun close() {
        if (opaque is kotlin.Array<*>) {
            val array = opaque as kotlin.Array<*>
            for (p in array) {
                if (p is Pointer) {
                    p.releaseReference()
                }
            }
            opaque = null
        }
    }
}
