package me.lampu.lampcord.shared.imaging

import coil3.decode.Decoder

expect fun platformImageDecoders(): List<Decoder.Factory>
