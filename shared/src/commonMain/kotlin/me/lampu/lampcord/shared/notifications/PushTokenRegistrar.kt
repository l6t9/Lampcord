package me.lampu.lampcord.shared.notifications

interface PushTokenRegistrar {
    fun register(token: String? = null)
}
