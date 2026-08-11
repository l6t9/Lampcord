package me.lampu.lampcord.shared.utils

import java.io.File
import com.sun.jna.Platform

object XdgUtils {
    fun getXdgConfigHome(): File {
        return when {
            Platform.isWindows() -> {
                val appData = System.getenv("APPDATA")
                if (appData != null) File(appData) else File(System.getProperty("user.home"), "AppData/Roaming")
            }
            Platform.isMac() -> {
                File(System.getProperty("user.home"), "Library/Application Support")
            }
            else -> {
                val env = System.getenv("XDG_CONFIG_HOME")
                if (env != null && env.isNotEmpty()) File(env) else File(System.getProperty("user.home"), ".config")
            }
        }
    }

    fun getXdgStateHome(): File {
        val env = System.getenv("XDG_STATE_HOME")
        return if (env != null && env.isNotEmpty()) {
            File(env)
        } else {
            File(System.getProperty("user.home"), ".local/state")
        }
    }
}
