package me.lampu.lampcord.shared.update

import me.lampu.lampcord.shared.utils.getPlatformName

fun currentUpdateTarget(): UpdateTarget? = when (getPlatformName()) {
    "android" -> UpdateTarget.ANDROID
    "linux" -> UpdateTarget.LINUX
    "windows" -> UpdateTarget.WINDOWS
    "macos" -> UpdateTarget.MACOS
    else -> null
}
