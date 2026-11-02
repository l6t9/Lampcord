package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
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
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.dismiss
import me.lampu.lampcord.shared.generated.resources.download
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_available_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_channel_nightly
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_channel_stable
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_check_for_updates
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_check_for_updates_desc
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_check_now
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_checking
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_downloading_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_install_restart
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_nightly_desc
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_ready_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_up_to_date
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_update_notifications
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_update_notifications_desc
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_version_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_view_changelog
import me.lampu.lampcord.shared.generated.resources.try_again
import me.lampu.lampcord.shared.generated.resources.whats_new
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.switchSettingsItem
import me.lampu.lampcord.shared.update.APP_VERSION
import me.lampu.lampcord.shared.update.InstallResult
import me.lampu.lampcord.shared.update.NIGHTLY_BUILD
import me.lampu.lampcord.shared.update.UpdateChannel
import me.lampu.lampcord.shared.update.UpdateManager
import me.lampu.lampcord.shared.update.UpdateState
import me.lampu.lampcord.shared.update.installUpdate
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

@Composable
fun UpdatesContent(
    updateManager: UpdateManager = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    modifier: Modifier = Modifier,
) {
    val state by updateManager.state.collectAsState()
    val scope = rememberCoroutineScope()

    val channel = if (NIGHTLY_BUILD > 0) UpdateChannel.NIGHTLY else UpdateChannel.STABLE
    var showChangelog by remember { mutableStateOf(false) }
    var installNote by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(channel) {
        if (settingsStore.checkForUpdates) updateManager.check(channel)
    }

    Column(
        modifier = modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        UpdateOverviewCard(
            version = APP_VERSION,
            channel = channel,
            state = state,
            installNote = installNote,
            onCheck = { scope.launch { updateManager.check(channel, force = true) } },
            onChangelog = { showChangelog = true },
            onDownload = { scope.launch { updateManager.download() } },
            onInstall = {
                val ready = state as? UpdateState.ReadyToInstall
                val target = me.lampu.lampcord.shared.update.currentUpdateTarget()
                if (ready != null && target != null) {
                    val result = installUpdate(
                        target = target,
                        downloadedFile = ready.file,
                        version = ready.version,
                    )
                    installNote = when (result) {
                        is InstallResult.Unsupported -> result.reason
                        is InstallResult.Failed -> result.reason
                        InstallResult.Started -> null
                    }
                }
            },
            onDismiss = { updateManager.reset() }
        )

        Material3SettingsGroup(
            items = listOf(
                switchSettingsItem(
                    title = stringResource(Res.string.lampcord_settings_check_for_updates),
                    description = stringResource(Res.string.lampcord_settings_check_for_updates_desc),
                    checked = settingsStore.checkForUpdates,
                    onCheckedChange = {
                        settingsStore.checkForUpdates = it
                        if (!it) updateManager.reset()
                    }
                ),
                switchSettingsItem(
                    title = stringResource(Res.string.lampcord_settings_update_notifications),
                    description = stringResource(Res.string.lampcord_settings_update_notifications_desc),
                    checked = settingsStore.updateNotifications,
                    onCheckedChange = { settingsStore.updateNotifications = it }
                )
            )
        )
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
private fun UpdateOverviewCard(
    version: String,
    channel: UpdateChannel,
    state: UpdateState,
    installNote: String?,
    onCheck: () -> Unit,
    onChangelog: () -> Unit,
    onDownload: () -> Unit,
    onInstall: () -> Unit,
    onDismiss: () -> Unit,
) {
    val nightly = channel == UpdateChannel.NIGHTLY
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(28.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Column(
            modifier = Modifier.padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    Text(
                        text = "Lampcord updates",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.SemiBold
                    )
                    Text(
                        text = stringResource(Res.string.lampcord_settings_version_fmt, version),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                Surface(
                    shape = CircleShape,
                    color = if (nightly) {
                        MaterialTheme.colorScheme.tertiaryContainer
                    } else {
                        MaterialTheme.colorScheme.secondaryContainer
                    },
                    contentColor = if (nightly) {
                        MaterialTheme.colorScheme.onTertiaryContainer
                    } else {
                        MaterialTheme.colorScheme.onSecondaryContainer
                    }
                ) {
                    Text(
                        text = stringResource(
                            if (nightly) {
                                Res.string.lampcord_settings_channel_nightly
                            } else {
                                Res.string.lampcord_settings_channel_stable
                            }
                        ),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Medium,
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp)
                    )
                }
            }

            if (nightly) {
                Text(
                    text = stringResource(Res.string.lampcord_settings_nightly_desc),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            when (val current = state) {
                is UpdateState.Checking -> BusyStatus(
                    title = stringResource(Res.string.lampcord_settings_checking),
                    subtitle = "Looking for the latest release."
                )

                is UpdateState.UpToDate -> {
                    Text(
                        text = stringResource(Res.string.lampcord_settings_up_to_date),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold
                    )
                    OutlinedButton(
                        onClick = onCheck,
                        shape = MaterialTheme.shapes.large
                    ) { Text(stringResource(Res.string.lampcord_settings_check_now)) }
                }

                is UpdateState.Idle -> {
                    Text(
                        text = "Check for a new release when you're ready.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Button(
                        onClick = onCheck,
                        shape = MaterialTheme.shapes.large
                    ) { Text(stringResource(Res.string.lampcord_settings_check_now)) }
                }

                is UpdateState.Available -> {
                    Text(
                        text = stringResource(Res.string.lampcord_settings_available_fmt, current.version),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold
                    )
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(
                            onClick = onDownload,
                            shape = MaterialTheme.shapes.large
                        ) { Text(stringResource(Res.string.download)) }
                        FilledTonalButton(
                            onClick = onChangelog,
                            shape = MaterialTheme.shapes.large
                        ) { Text(stringResource(Res.string.whats_new)) }
                    }
                    TextButton(onClick = onDismiss) { Text(stringResource(Res.string.dismiss)) }
                }

                is UpdateState.Downloading -> {
                    BusyStatus(
                        title = stringResource(Res.string.lampcord_settings_downloading_fmt, current.version),
                        subtitle = "This can take a moment."
                    )
                    LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
                }

                is UpdateState.ReadyToInstall -> {
                    Text(
                        text = stringResource(Res.string.lampcord_settings_ready_fmt, current.version),
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold
                    )
                    installNote?.let {
                        Text(
                            text = it,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.error
                        )
                    }
                    Button(
                        onClick = onInstall,
                        shape = MaterialTheme.shapes.large
                    ) { Text(stringResource(Res.string.lampcord_settings_install_restart)) }
                }

                is UpdateState.Failed -> {
                    Text(
                        text = current.reason,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.error
                    )
                    OutlinedButton(
                        onClick = onCheck,
                        shape = MaterialTheme.shapes.large
                    ) { Text(stringResource(Res.string.try_again)) }
                }
            }
        }
    }
}

@Composable
private fun BusyStatus(title: String, subtitle: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        ContainedLoadingIndicator(modifier = Modifier.size(40.dp))
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}
