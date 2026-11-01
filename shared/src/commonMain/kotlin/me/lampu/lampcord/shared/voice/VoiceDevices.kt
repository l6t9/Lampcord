package me.lampu.lampcord.shared.voice

data class VoiceDevice(
    val id: String,
    val name: String
)

expect fun listVoiceInputDevices(): List<VoiceDevice>
expect fun listVoiceOutputDevices(): List<VoiceDevice>
