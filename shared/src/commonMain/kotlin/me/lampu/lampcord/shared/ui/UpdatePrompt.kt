package me.lampu.lampcord.shared.ui

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
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.update.InstallResult
import me.lampu.lampcord.shared.update.NIGHTLY_BUILD
import me.lampu.lampcord.shared.update.UpdateChannel
import me.lampu.lampcord.shared.update.UpdateManager
import me.lampu.lampcord.shared.update.UpdateState
import me.lampu.lampcord.shared.update.UpdateTarget
import me.lampu.lampcord.shared.update.currentUpdateTarget
import me.lampu.lampcord.shared.update.installUpdate
import me.lampu.lampcord.shared.ui.components.DiscordBottomSheet
import me.lampu.lampcord.shared.ui.components.rememberDiscordSheetState
import org.koin.compose.koinInject

@Composable
fun UpdatePrompt(
    updateManager: UpdateManager = koinInject(),
    settingsStore: SettingsStore = koinInject(),
) {
    if (currentUpdateTarget() != UpdateTarget.ANDROID) return

    val state by updateManager.state.collectAsState()
    val scope = rememberCoroutineScope()
    var dismissed by rememberSaveable { mutableStateOf(false) }
    var installNote by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        if (!settingsStore.checkForUpdates) return@LaunchedEffect
        val channel = if (NIGHTLY_BUILD > 0) UpdateChannel.NIGHTLY else UpdateChannel.STABLE
        updateManager.check(channel)
    }

    val available = state as? UpdateState.Available
    val ready = state as? UpdateState.ReadyToInstall

    if (available != null && !dismissed && settingsStore.updateNotifications) {
        UpdateSheet(
            title = "Update available",
            version = available.version,
            notes = available.changelog,
            primaryLabel = "Update",
            secondaryLabel = "Later",
            busyLabel = null,
            progress = null,
            footnote = installNote,
            onPrimary = { scope.launch { updateManager.download() } },
            onSecondary = { dismissed = true },
        )
    } else if (ready != null) {
        UpdateSheet(
            title = "Update ready",
            version = ready.version,
            notes = "",
            primaryLabel = "Install and restart",
            secondaryLabel = "Not now",
            busyLabel = null,
            progress = null,
            footnote = installNote,
            onPrimary = {
                val result = installUpdate(
                    target = UpdateTarget.ANDROID,
                    downloadedFile = ready.file,
                    version = ready.version,
                )
                installNote = when (result) {
                    is InstallResult.Unsupported -> result.reason
                    is InstallResult.Failed -> result.reason
                    InstallResult.Started -> null
                }
            },
            onSecondary = { dismissed = true },
        )
    } else if (state is UpdateState.Downloading) {
        UpdateSheet(
            title = "Downloading update",
            version = "",
            notes = "",
            primaryLabel = null,
            secondaryLabel = null,
            busyLabel = (state as? UpdateState.Downloading)?.version,
            progress = 0f,
            footnote = null,
            onPrimary = {},
            onSecondary = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun UpdateSheet(
    title: String,
    version: String,
    notes: String,
    primaryLabel: String?,
    secondaryLabel: String?,
    busyLabel: String?,
    progress: Float?,
    footnote: String?,
    onPrimary: () -> Unit,
    onSecondary: () -> Unit,
) {
    DiscordBottomSheet(
        onDismissRequest = if (progress != null) ({}) else onSecondary,
        sheetState = rememberDiscordSheetState(),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = 520.dp)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 24.dp)
                .padding(bottom = 32.dp),
        ) {
            Text(
                title,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
            )
            if (version.isNotBlank()) {
                Text(
                    "Version $version",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 4.dp, bottom = 12.dp),
                )
            }
            when {
                progress != null -> {
                    busyLabel?.let {
                        Text(
                            it,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(bottom = 8.dp),
                        )
                    }
                    LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
                }

                notes.isNotBlank() -> Text(notes, style = MaterialTheme.typography.bodyMedium)

                else -> CircularProgressIndicator(modifier = Modifier.heightIn(max = 18.dp))
            }
            footnote?.let {
                Text(
                    it,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.error,
                    modifier = Modifier.padding(top = 12.dp),
                )
            }
            if (primaryLabel != null || secondaryLabel != null) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 20.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    secondaryLabel?.let {
                        TextButton(onClick = onSecondary) { Text(it) }
                    }
                    primaryLabel?.let {
                        Button(onClick = onPrimary) { Text(it) }
                    }
                }
            }
        }
    }
}