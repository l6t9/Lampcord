package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.update.APP_VERSION
import me.lampu.lampcord.shared.update.InstallResult
import me.lampu.lampcord.shared.update.UpdateChannel
import me.lampu.lampcord.shared.update.UpdateManager
import me.lampu.lampcord.shared.update.UpdateState
import me.lampu.lampcord.shared.update.installUpdate
import org.koin.compose.koinInject

@Composable
fun UpdatesContent(
    updateManager: UpdateManager = koinInject(),
    modifier: Modifier = Modifier,
) {
    val state by updateManager.state.collectAsState()
    val scope = rememberCoroutineScope()
    var channel by remember { mutableStateOf(UpdateChannel.STABLE) }
    var showChangelog by remember { mutableStateOf(false) }
    var installNote by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(channel) { updateManager.check(channel) }

    Column(
        modifier = modifier
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text(
            "Updates",
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold,
        )

        Text("Version $APP_VERSION", style = MaterialTheme.typography.bodyMedium)

        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
            UpdateChannel.entries.forEachIndexed { index, entry ->
                SegmentedButton(
                    selected = channel == entry,
                    onClick = { channel = entry },
                    shape = SegmentedButtonDefaults.itemShape(index, UpdateChannel.entries.size),
                ) {
                    Text(if (entry == UpdateChannel.STABLE) "Stable" else "Nightly")
                }
            }
        }

        if (channel == UpdateChannel.NIGHTLY) {
            Text(
                "Nightly builds come from the latest commit to main and can be unstable.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        when (val current = state) {
            is UpdateState.Checking -> StatusRow("Checking for updates", busy = true)
            is UpdateState.UpToDate -> StatusRow("Lampcord is up to date")
            is UpdateState.Idle -> Button(
                onClick = { scope.launch { updateManager.check(channel, force = true) } },
            ) { Text("Check for updates") }

            is UpdateState.Available -> Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(
                    "Version ${current.version} is available",
                    style = MaterialTheme.typography.titleMedium,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Button(onClick = { showChangelog = true }) { Text("What's new") }
                    Button(onClick = { scope.launch { updateManager.download() } }) {
                        Text("Download")
                    }
                }
                TextButton(onClick = { updateManager.reset() }) { Text("Dismiss") }
            }

            is UpdateState.Downloading -> Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Downloading ${current.version}")
                LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            }

            is UpdateState.ReadyToInstall -> Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("${current.version} is ready to install")
                installNote?.let {
                    Text(
                        it,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Button(
                    onClick = {
                        val result = installUpdate(
                            target = me.lampu.lampcord.shared.update.currentUpdateTarget()
                                ?: return@Button,
                            downloadedFile = current.file,
                            version = current.version,
                        )
                        installNote = when (result) {
                            is InstallResult.Unsupported -> result.reason
                            is InstallResult.Failed -> result.reason
                            InstallResult.Started -> null
                        }
                    }
                ) { Text("Install and restart") }
            }

            is UpdateState.Failed -> Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    current.reason,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.error,
                )
                OutlinedButton(
                    onClick = { scope.launch { updateManager.check(channel, force = true) } }
                ) { Text("Try again") }
            }
        }
    }

    if (showChangelog) {
        val available = state as? UpdateState.Available
        ChangelogSheet(
            version = available?.release?.version.orEmpty(),
            notes = available?.changelog.orEmpty(),
            onDismiss = { showChangelog = false },
        )
    }
}

@Composable
private fun StatusRow(text: String, busy: Boolean = false) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        if (busy) CircularProgressIndicator(modifier = Modifier.heightIn(max = 18.dp))
        Text(text, style = MaterialTheme.typography.bodyMedium)
    }
}
