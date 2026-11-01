package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.components.settings.expandableSettingsItem
import me.lampu.lampcord.shared.ui.components.settings.switchSettingsItem
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import me.lampu.lampcord.shared.voice.VoiceDevice
import me.lampu.lampcord.shared.voice.listVoiceInputDevices
import me.lampu.lampcord.shared.voice.listVoiceOutputDevices
import kotlin.math.roundToInt

@Composable
fun VoiceSettings(
    onBack: () -> Unit
) {
    SettingsSubScreen(
        title = "Voice & Video",
        onNavigateBack = onBack
    ) {
        VoiceSettingsContent()
    }
}

@Composable
fun VoiceSettingsContent() {
    val inputDevices = remember { listVoiceInputDevices().ifEmpty { listOf(VoiceDevice("", "System default")) } }
    val outputDevices = remember { listVoiceOutputDevices().ifEmpty { listOf(VoiceDevice("", "System default")) } }
    var showInputs by remember { mutableStateOf(false) }
    var showOutputs by remember { mutableStateOf(false) }

    val selectedInput = inputDevices.firstOrNull { it.id == Settings.shared.voiceInputDevice }
        ?: inputDevices.firstOrNull()
    val selectedOutput = outputDevices.firstOrNull { it.id == Settings.shared.voiceOutputDevice }
        ?: outputDevices.firstOrNull()

    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "Input",
            items = listOf(
                expandableSettingsItem(
                    title = "Input Device",
                    description = selectedInput?.name ?: "System default",
                    expanded = showInputs,
                    onToggle = { showInputs = !showInputs }
                ) {
                    Column {
                        inputDevices.forEach { device ->
                            DeviceChoice(
                                device = device,
                                selected = device.id == (selectedInput?.id ?: ""),
                                onSelect = {
                                    Settings.shared.voiceInputDevice = device.id
                                    showInputs = false
                                }
                            )
                        }
                    }
                }
            )
        )

        Material3SettingsGroup(
            title = "Output",
            items = listOf(
                expandableSettingsItem(
                    title = "Output Device",
                    description = selectedOutput?.name ?: "System default",
                    expanded = showOutputs,
                    onToggle = { showOutputs = !showOutputs }
                ) {
                    Column {
                        outputDevices.forEach { device ->
                            DeviceChoice(
                                device = device,
                                selected = device.id == (selectedOutput?.id ?: ""),
                                onSelect = {
                                    Settings.shared.voiceOutputDevice = device.id
                                    showOutputs = false
                                }
                            )
                        }
                    }
                }
            )
        )

        Material3SettingsGroup(title = "Volume") {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                VolumeSlider(
                    label = "Input Volume",
                    value = Settings.shared.voiceInputVolume,
                    onChange = { Settings.shared.voiceInputVolume = it }
                )
                VolumeSlider(
                    label = "Output Volume",
                    value = Settings.shared.voiceOutputVolume,
                    onChange = { Settings.shared.voiceOutputVolume = it }
                )
            }
        }

        Material3SettingsGroup(
            title = "Processing",
            items = listOf(
                switchSettingsItem(
                    title = "Noise Suppression",
                    description = "Reduce background noise before your microphone is sent.",
                    checked = Settings.shared.noiseCancellation,
                    onCheckedChange = { Settings.shared.noiseCancellation = it }
                )
            )
        )
    }
}

@Composable
private fun DeviceChoice(
    device: VoiceDevice,
    selected: Boolean,
    onSelect: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickableCursor(onClick = onSelect)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = device.name,
            style = MaterialTheme.typography.bodyMedium,
            color = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f)
        )
        if (selected) {
            Spacer(Modifier.width(8.dp))
            Icon(
                imageVector = Icons.Filled.Check,
                contentDescription = null,
                modifier = Modifier.size(18.dp),
                tint = MaterialTheme.colorScheme.primary
            )
        }
    }
}

@Composable
private fun VolumeSlider(
    label: String,
    value: Float,
    onChange: (Float) -> Unit
) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.width(112.dp)
        )
        Slider(
            value = value,
            onValueChange = onChange,
            valueRange = 0f..2f,
            modifier = Modifier.weight(1f)
        )
        Text(
            text = "${(value * 100).roundToInt()}%",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.width(48.dp)
        )
    }
}
