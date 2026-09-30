package me.lampu.lampcord.shared.voice

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.*
import androidx.compose.ui.platform.LocalContext
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.notifications.IncomingCallNotifier
import me.lampu.lampcord.shared.state.VoiceStore
import me.lampu.lampcord.shared.utils.AndroidContextProvider

@Composable
actual fun rememberVoiceJoin(voiceStore: VoiceStore): (Channel, Boolean) -> Unit {
    val context = LocalContext.current
    var pending by remember { mutableStateOf<Pair<Channel, Boolean>?>(null) }
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
        val request = pending
        pending = null
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED && request != null) {
            voiceStore.connectToVoice(request.first, request.second)
        } else voiceStore.error = "Microphone permission is required for voice calls. Enable it in Android app settings."
    }
    return { channel, ring ->
        val preferences = context.getSharedPreferences("voice_permissions", 0)
        val askBluetooth = Build.VERSION.SDK_INT >= 31 &&
            context.checkSelfPermission(Manifest.permission.BLUETOOTH_CONNECT) != PackageManager.PERMISSION_GRANTED &&
            !preferences.getBoolean("bluetooth_asked", false)
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED && !askBluetooth) {
            voiceStore.connectToVoice(channel, ring)
        } else {
            pending = channel to ring
            if (askBluetooth) preferences.edit().putBoolean("bluetooth_asked", true).apply()
            launcher.launch(buildList {
                add(Manifest.permission.RECORD_AUDIO)
                if (askBluetooth) add(Manifest.permission.BLUETOOTH_CONNECT)
            }.toTypedArray())
        }
    }
}

actual fun startVoiceSession() {
    val context = AndroidContextProvider.applicationContext
    check(context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED)
    val intent = Intent().setClassName(context.packageName, "me.lampu.lampcord.VoiceCallService")
    if (Build.VERSION.SDK_INT >= 26) context.startForegroundService(intent) else context.startService(intent)
}

actual fun stopVoiceSession() {
    val context = AndroidContextProvider.applicationContext
    context.stopService(Intent().setClassName(context.packageName, "me.lampu.lampcord.VoiceCallService"))
}

actual fun notifyIncomingCall(channelId: String?) =
    IncomingCallNotifier.notifyIncomingCall(channelId)
