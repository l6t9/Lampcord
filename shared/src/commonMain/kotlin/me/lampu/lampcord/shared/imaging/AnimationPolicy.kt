package me.lampu.lampcord.shared.imaging

import coil3.Extras
import coil3.request.Options

internal val ALLOW_ANIMATION_KEY = Extras.Key(default = true)

internal fun Options.allowsAnimation(): Boolean = extras[ALLOW_ANIMATION_KEY] ?: true
