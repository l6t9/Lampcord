package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.DiscordUrl
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun JoinServerDialog(
    onDismiss: () -> Unit,
    guildApi: GuildApi = koinInject(),
    onJoined: (Guild) -> Unit = {}
) {
    var input by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    suspend fun join() {
        val code = extractInviteCode(input)
        if (code == null) {
            error = "That does not look like an invite link."
            return
        }
        busy = true
        error = null
        val guild = guildApi.joinGuild(code)
        busy = false
        if (guild != null) {
            onJoined(guild)
            onDismiss()
        } else {
            error = "That invite is invalid, expired, or you are already in that server."
        }
    }

    AlertDialog(
        onDismissRequest = { if (!busy) onDismiss() },
        title = { Text("Join a Server") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "Paste an invite link below.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                OutlinedTextField(
                    value = input,
                    onValueChange = { input = it; error = null },
                    placeholder = { Text("discord.gg/example", maxLines = 1, overflow = TextOverflow.Ellipsis) },
                    singleLine = true,
                    enabled = !busy,
                    isError = error != null,
                    supportingText = { error?.let { Text(it) } },
                    keyboardOptions = KeyboardOptions(
                        capitalization = KeyboardCapitalization.None,
                        keyboardType = KeyboardType.Uri,
                        imeAction = ImeAction.Done
                    )
                )
            }
        },
        confirmButton = {
            TextButton(
                enabled = !busy && input.isNotBlank(),
                onClick = { scope.launch { join() } }
            ) {
                if (busy) {
                    CircularProgressIndicator(Modifier.size(16.dp), strokeWidth = 2.dp)
                    Spacer(Modifier.width(8.dp))
                }
                Text("Join")
            }
        },
        dismissButton = { TextButton(enabled = !busy, onClick = onDismiss) { Text("Cancel") } }
    )
}

fun extractInviteCode(input: String): String? {
    val trimmed = input.trim()
    if (trimmed.isEmpty()) return null
    if (!trimmed.contains('.')) {
        return trimmed.substringAfterLast('/').takeIf { it.isNotEmpty() && !it.contains(' ') }
    }
    val target = DiscordUrl.parse(trimmed)
    if (target is DiscordUrl.Target.Invite) return target.code
    return trimmed.substringBefore('?').trimEnd('/').substringAfterLast('/')
        .takeIf { it.isNotEmpty() && it.none { c -> c.isWhitespace() || c == '<' || c == '>' } }
}

@Composable
fun AddServerRailItem(onClick: () -> Unit) {
    Box(modifier = Modifier.size(48.dp), contentAlignment = Alignment.Center) {
        RegularGuildItem(
            isSelected = false,
            isMonogram = true,
            onClick = onClick,
            selectedColor = MaterialTheme.colorScheme.primary,
            unselectedColor = MaterialTheme.colorScheme.surfaceVariant,
            monogramSelectedColor = MaterialTheme.colorScheme.onPrimary,
            monogramUnselectedColor = MaterialTheme.colorScheme.primary
        ) {
            Icon(
                imageVector = Icons.Rounded.Add,
                contentDescription = "Join a server",
                modifier = Modifier.size(28.dp)
            )
        }
    }
}