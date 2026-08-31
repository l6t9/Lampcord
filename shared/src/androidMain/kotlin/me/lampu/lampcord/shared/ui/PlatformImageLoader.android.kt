package me.lampu.lampcord.shared.ui

import android.os.Build
import coil3.ImageLoader
import coil3.gif.AnimatedImageDecoder
import coil3.gif.GifDecoder

actual fun addPlatformImageDecoders(builder: ImageLoader.Builder): ImageLoader.Builder =
    builder.components {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            add(AnimatedImageDecoder.Factory())
        } else {
            add(GifDecoder.Factory())
        }
    }
