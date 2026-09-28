package me.lampu.lampcord.shared.imaging

import coil3.decode.Decoder

actual fun platformImageDecoders(): List<Decoder.Factory> =
    listOf(DesktopAnimatedDecoder.Factory(), DesktopStillDecoder.Factory())
