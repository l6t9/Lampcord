package me.lampu.lampcord.shared.voice

import android.media.AudioDeviceInfo
import android.media.AudioManager
import me.lampu.lampcord.shared.utils.AndroidContextProvider

actual fun listVoiceInputDevices(): List<VoiceDevice> =
    listVoiceDevices(AudioManager.GET_DEVICES_INPUTS)

actual fun listVoiceOutputDevices(): List<VoiceDevice> =
    listVoiceDevices(AudioManager.GET_DEVICES_OUTPUTS)

private fun listVoiceDevices(direction: Int): List<VoiceDevice> = buildList {
    add(VoiceDevice("", "System default"))
    val manager = AndroidContextProvider.applicationContext.getSystemService(AudioManager::class.java)
    manager.getDevices(direction)
        .filter { it.type != AudioDeviceInfo.TYPE_TELEPHONY }
        .forEach { device ->
            val name = device.productName?.toString()?.takeIf { it.isNotBlank() }
                ?: deviceTypeName(device.type)
            add(VoiceDevice(device.id.toString(), name))
        }
}

private fun deviceTypeName(type: Int): String = when (type) {
    AudioDeviceInfo.TYPE_BUILTIN_EARPIECE -> "Earpiece"
    AudioDeviceInfo.TYPE_BUILTIN_SPEAKER -> "Speaker"
    AudioDeviceInfo.TYPE_BUILTIN_MIC -> "Microphone"
    AudioDeviceInfo.TYPE_WIRED_HEADSET -> "Wired headset"
    AudioDeviceInfo.TYPE_WIRED_HEADPHONES -> "Wired headphones"
    AudioDeviceInfo.TYPE_BLUETOOTH_SCO -> "Bluetooth"
    AudioDeviceInfo.TYPE_BLUETOOTH_A2DP -> "Bluetooth audio"
    AudioDeviceInfo.TYPE_BLE_HEADSET -> "Bluetooth LE headset"
    AudioDeviceInfo.TYPE_USB_DEVICE -> "USB device"
    AudioDeviceInfo.TYPE_USB_HEADSET -> "USB headset"
    else -> "Audio device"
}
