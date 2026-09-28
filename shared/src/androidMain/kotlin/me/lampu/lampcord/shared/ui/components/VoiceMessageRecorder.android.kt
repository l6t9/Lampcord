package me.lampu.lampcord.shared.ui.components

import android.Manifest
import android.content.pm.PackageManager
import android.media.MediaRecorder
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.ui.icons.Icons
import java.io.File

@Composable
actual fun VoiceMessageRecorder(
    enabled: Boolean,
    buttonSize: Dp,
    iconSize: Dp,
    modifier: Modifier,
    onRecordingReady: (PendingFile) -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val recorderState = remember { mutableStateOf<MediaRecorder?>(null) }
    val outputFileState = remember { mutableStateOf<File?>(null) }
    var isRecording by remember { mutableStateOf(false) }
    var elapsedSeconds by remember { mutableIntStateOf(0) }

    fun showRecordingError() {
        android.widget.Toast.makeText(
            context,
            "Could not record a voice message",
            android.widget.Toast.LENGTH_SHORT
        ).show()
    }

    fun startRecording() {
        if (recorderState.value != null) return

        val useOggOpus = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q
        val extension = if (useOggOpus) "ogg" else "aac"
        val file = File(context.cacheDir, "lampcord_voice_${System.currentTimeMillis()}.$extension")
        try {
            val recorder = MediaRecorder(context).apply {
                setAudioSource(MediaRecorder.AudioSource.MIC)
                if (useOggOpus) {
                    setOutputFormat(MediaRecorder.OutputFormat.OGG)
                    setAudioEncoder(MediaRecorder.AudioEncoder.OPUS)
                    setAudioSamplingRate(48_000)
                    setAudioEncodingBitRate(64_000)
                } else {
                    setOutputFormat(MediaRecorder.OutputFormat.AAC_ADTS)
                    setAudioEncoder(MediaRecorder.AudioEncoder.AAC)
                    setAudioSamplingRate(44_100)
                    setAudioEncodingBitRate(128_000)
                }
                setOutputFile(file.absolutePath)
                prepare()
                start()
            }
            outputFileState.value = file
            recorderState.value = recorder
            elapsedSeconds = 0
            isRecording = true
        } catch (_: Exception) {
            recorderState.value?.release()
            recorderState.value = null
            file.delete()
            showRecordingError()
        }
    }

    fun stopRecording() {
        val recorder = recorderState.value ?: return
        val file = outputFileState.value
        recorderState.value = null
        outputFileState.value = null
        isRecording = false
        elapsedSeconds = 0

        val stopped = runCatching { recorder.stop() }.isSuccess
        recorder.release()
        if (!stopped || file == null) {
            file?.delete()
            showRecordingError()
            return
        }

        scope.launch(Dispatchers.IO) {
            val bytes = runCatching { file.readBytes() }.getOrNull() ?: ByteArray(0)
            file.delete()
            if (bytes.isNotEmpty()) {
                withContext(Dispatchers.Main.immediate) {
                    onRecordingReady(
                        PendingFile(
                            name = "voice_message_${System.currentTimeMillis()}.${file.extension}",
                            data = bytes
                        )
                    )
                }
            } else {
                withContext(Dispatchers.Main.immediate) {
                    showRecordingError()
                }
            }
        }
    }

    val permissionLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { granted ->
        if (granted) startRecording() else showRecordingError()
    }

    fun toggleRecording() {
        if (isRecording) {
            stopRecording()
        } else if (ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED) {
            startRecording()
        } else {
            permissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
        }
    }

    LaunchedEffect(isRecording) {
        while (isRecording) {
            delay(1_000)
            elapsedSeconds++
        }
    }

    DisposableEffect(Unit) {
        onDispose {
            recorderState.value?.runCatching { stop() }
            recorderState.value?.release()
            recorderState.value = null
            outputFileState.value?.delete()
            outputFileState.value = null
        }
    }

    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically
    ) {
        FilledIconButton(
            onClick = ::toggleRecording,
            enabled = enabled,
            modifier = Modifier.size(buttonSize),
            colors = if (isRecording) {
                IconButtonDefaults.filledIconButtonColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer,
                    contentColor = MaterialTheme.colorScheme.onErrorContainer
                )
            } else {
                IconButtonDefaults.filledIconButtonColors(
                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        ) {
            Icon(
                imageVector = if (isRecording) Icons.Filled.Close else Icons.Filled.Mic,
                contentDescription = if (isRecording) "Stop recording" else "Record voice message",
                modifier = Modifier.size(iconSize)
            )
        }
        if (isRecording) {
            Text(
                text = "%02d:%02d".format(elapsedSeconds / 60, elapsedSeconds % 60),
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.width(38.dp)
            )
        }
    }
}
