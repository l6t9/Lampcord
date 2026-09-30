package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlin.time.Instant
import me.lampu.lampcord.shared.api.DeviceIcon
import me.lampu.lampcord.shared.api.DeviceSession
import me.lampu.lampcord.shared.api.SessionApi
import me.lampu.lampcord.shared.api.SessionFailure
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.ui.components.settings.SettingsSubScreen
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import kotlin.time.Clock

private data class Verification(
    val title: String,
    val hint: String,
    val password: Boolean = false,
    val numeric: Boolean = false,
    val allowBackup: Boolean = false,
    val onConfirm: suspend (String) -> Unit
)

/**
 * Drives the device list and the escalation Discord asks for when signing a session out: a
 * password, then an authenticator code or a backup code.
 *
 * Each step hands the collected credential straight to the next request rather than storing it,
 * so a cancelled dialog leaves nothing behind.
 */
private class DevicesController(
    private val api: SessionApi,
    private val scope: CoroutineScope
) {
    var sessions by mutableStateOf<List<DeviceSession>?>(null)
        private set
    var status by mutableStateOf("Loading devices…")
        private set
    var busy by mutableStateOf(false)
        private set
    var verification by mutableStateOf<Verification?>(null)
        private set

    val current: List<DeviceSession> get() = sessions.orEmpty().filter { it.current }
    val others: List<DeviceSession> get() = sessions.orEmpty().filterNot { it.current }

    fun refresh() = scope.launch {
        busy = true
        status = "Loading devices…"
        runCatching { api.listSessions() }
            .onSuccess {
                sessions = it
                status = ""
            }
            .onFailure { status = "Could not load devices: ${it.message?.take(160)}" }
        busy = false
    }

    fun logout(target: DeviceSession?, password: String? = null, mfaToken: String? = null, backupCode: String? = null) =
        scope.launch {
            val fallback = sessions.orEmpty().filterNot { it.current }.map { it.idHash }
            val single = target?.idHash?.takeIf { it.isNotEmpty() }
            val hashes = if (single != null) listOf(single) else fallback
            if (hashes.isEmpty()) {
                status = "There are no other devices to log out."
                return@launch
            }
            busy = true
            status = "Logging out…"
            runCatching { api.logout(hashes, password, mfaToken, backupCode) }
                .onSuccess {
                    verification = null
                    refresh()
                }
                .onFailure { error -> escalate(error, target, password, mfaToken, backupCode) }
            busy = false
        }

    private suspend fun escalate(
        error: Throwable,
        target: DeviceSession?,
        password: String?,
        mfaToken: String?,
        backupCode: String?
    ) {
        if (error !is SessionFailure) {
            status = "Could not log out device: ${error.message?.take(160)}"
            return
        }
        val challenge = error.mfa
        when {
            challenge != null && mfaToken == null -> askMfa(challenge, target, password)
            error.requiresPassword && password == null -> verification = Verification(
                title = "Confirm with your password",
                hint = "Password",
                password = true
            ) { value -> logout(target, password = value) }

            error.requiresCode && backupCode == null -> verification = Verification(
                title = "Verify with a backup code",
                hint = "Backup code"
            ) { value -> logout(target, password, backupCode = value) }

            else -> status = "Could not log out device: ${error.message?.take(160)}"
        }
    }

    private fun askMfa(challenge: JsonObject, target: DeviceSession?, password: String?) {
        val ticket = challenge["ticket"].text() ?: run {
            status = "Discord requested 2FA without a challenge ticket."
            return
        }
        val methods = (challenge["methods"] as? JsonArray)
            ?.mapNotNull { element ->
                when (element) {
                    is JsonPrimitive -> element.content
                    is JsonObject -> element["type"].text()
                    else -> null
                }
            }
            ?.toSet()
        if (methods == null || "totp" in methods) {
            verification = Verification(
                title = "Verify with your authenticator app",
                hint = "6-digit authentication code",
                numeric = true,
                allowBackup = methods == null || "backup" in methods
            ) { code -> finishMfa(ticket, "totp", code, target, password) }
        } else if ("backup" in methods) {
            verification = Verification("Verify with a backup code", "Backup code") { code ->
                logout(target, password, backupCode = code)
            }
        } else {
            status = "Discord requested an unsupported verification method."
        }
    }

    private fun finishMfa(
        ticket: String,
        type: String,
        code: String,
        target: DeviceSession?,
        password: String?
    ) = scope.launch {
        busy = true
        status = "Verifying code…"
        runCatching { api.finishMfa(ticket, type, code) }
            .onSuccess { token -> logout(target, password, mfaToken = token) }
            .onFailure { status = "Could not verify code: ${it.message?.take(160)}" }
        busy = false
    }

    fun dismissVerification() {
        verification = null
    }
}

@Composable
fun DevicesSettings(onBack: () -> Unit) {
    SettingsSubScreen(
        title = "Devices",
        onNavigateBack = onBack
    ) {
        DevicesSettingsContent()
    }
}

@Composable
fun DevicesSettingsContent(sessionApi: SessionApi = koinInject()) {
    val scope = rememberCoroutineScope()
    val controller = remember { DevicesController(sessionApi, scope) }

    LaunchedEffect(Unit) { controller.refresh() }

    controller.verification?.let { pending ->
        VerificationSheet(
            verification = pending,
            onDismiss = { controller.dismissVerification() },
            onConfirm = { value ->
                controller.dismissVerification()
                scope.launch { pending.onConfirm(value) }
            },
            onAlternate = if (pending.allowBackup) {
                {
                    controller.dismissVerification()
                    controller.logout(null, backupCode = null)
                }
            } else null
        )
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = "Here are all the devices that are currently logged in with your Discord account. " +
                "You can log out of each one individually or all other devices.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )

        if (controller.status.isNotEmpty()) {
            Text(
                text = controller.status,
                style = MaterialTheme.typography.bodyMedium,
                color = if (controller.busy) {
                    MaterialTheme.colorScheme.onSurfaceVariant
                } else {
                    MaterialTheme.colorScheme.error
                },
                modifier = Modifier
                    .then(
                        if (!controller.busy) Modifier.clickable { controller.refresh() } else Modifier
                    )
                    .padding(horizontal = 16.dp, vertical = 8.dp)
            )
        }

        Material3SettingsGroup(
            title = "Current device",
            items = if (controller.current.isEmpty()) {
                listOf(placeholderItem("No device found"))
            } else {
                controller.current.map { it.toItem(controller) }
            }
        )

        Material3SettingsGroup(
            title = "Other devices",
            items = if (controller.sessions != null && controller.others.isEmpty()) {
                listOf(placeholderItem("No other active sessions found"))
            } else {
                controller.others.map { it.toItem(controller) }
            }
        )

        if (controller.others.isNotEmpty()) {
            Material3SettingsGroup(
                title = "Security",
                items = listOf(
                    Material3SettingsItem(
                        title = { Text("Log Out All Other Sessions", color = MaterialTheme.colorScheme.error) },
                        description = { Text("You'll have to log back in on all logged out devices.") },
                        onClick = { controller.logout(null) }
                    )
                )
            )
        }

        Material3SettingsGroup(
            title = null,
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Some older devices may not be shown here", color = MaterialTheme.colorScheme.primary) }
                ),
                Material3SettingsItem(
                    title = { Text("To log them out, change your password", color = MaterialTheme.colorScheme.primary) }
                )
            )
        )
    }
}

private fun DeviceSession.toItem(controller: DevicesController) = Material3SettingsItem(
    icon = if (icon == DeviceIcon.Mobile) Icons.Rounded.Devices else Icons.Rounded.Tv,
    title = { Text(name) },
    description = {
        Text(
            buildString {
                append(location ?: "Unknown location")
                if (!current && lastUsed != null) {
                    append(" • ")
                    append(relativeTime(lastUsed))
                }
            }
        )
    },
    trailingContent = {
        if (!current) {
            TextButton(enabled = !controller.busy, onClick = { controller.logout(this@toItem) }) {
                Text("Log Out", color = MaterialTheme.colorScheme.error)
            }
        }
    }
)

private fun placeholderItem(text: String) = Material3SettingsItem(
    title = { Text(text, color = MaterialTheme.colorScheme.onSurfaceVariant) }
)

private fun relativeTime(value: String): String {
    val then = runCatching { Instant.parse(value).toEpochMilliseconds() }.getOrNull() ?: return value
    val age = Clock.System.now().toEpochMilliseconds() - then
    return when {
        age < 3_600_000 -> "less than an hour ago"
        age < 86_400_000 -> "${age / 3_600_000} hours ago"
        else -> "${age / 86_400_000} days ago"
    }
}

private fun kotlinx.serialization.json.JsonElement?.text(): String? {
    val primitive = this as? JsonPrimitive ?: return null
    if (!primitive.isString) return null
    return primitive.content.takeIf { it.isNotBlank() && it != "null" }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun VerificationSheet(
    verification: Verification,
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit,
    onAlternate: (() -> Unit)?
) {
    var value by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(verification.title) },
        text = {
            Column {
                OutlinedTextField(
                    value = value,
                    onValueChange = { next ->
                        error = null
                        value = if (verification.numeric) next.filter { it.isDigit() }.take(6) else next
                    },
                    label = { Text(verification.hint) },
                    singleLine = true,
                    isError = error != null,
                    supportingText = { error?.let { Text(it) } },
                    visualTransformation = if (verification.password) {
                        PasswordVisualTransformation()
                    } else {
                        VisualTransformation.None
                    },
                    keyboardOptions = KeyboardOptions(
                        keyboardType = when {
                            verification.password -> KeyboardType.Password
                            verification.numeric -> KeyboardType.Number
                            else -> KeyboardType.Text
                        }
                    )
                )
                if (onAlternate != null) {
                    TextButton(onClick = onAlternate) { Text("Use a backup code") }
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    when {
                        value.isEmpty() -> error = "Required"
                        verification.numeric && value.length != 6 ->
                            error = "Enter a 6-digit authentication code"

                        else -> onConfirm(value.trim())
                    }
                }
            ) { Text("Confirm") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } }
    )
}