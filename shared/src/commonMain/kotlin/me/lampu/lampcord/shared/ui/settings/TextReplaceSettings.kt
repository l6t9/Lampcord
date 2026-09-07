package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.TextReplaceRule
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun TextReplaceSettings(settingsStore: SettingsStore = koinInject()) {
    var showAddDialog by remember { mutableStateOf(false) }
    var editingRule by remember { mutableStateOf<TextReplaceRule?>(null) }

    Column(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Material3SettingsGroup(title = "Rules") {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    "Automatically replace text patterns in messages.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(Modifier.height(16.dp))
                Button(
                    onClick = { showAddDialog = true },
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Icon(Icons.Filled.Add, null)
                    Spacer(Modifier.width(8.dp))
                    Text("Add New Rule")
                }
            }
        }

        if (settingsStore.textReplaceRules.isEmpty()) {
            Box(Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                Text("No replacement rules yet.", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            settingsStore.textReplaceRules.forEach { rule ->
                RuleItem(
                    rule = rule,
                    onEdit = { editingRule = it },
                    onToggle = { enabled ->
                        val updated = settingsStore.textReplaceRules.map {
                            if (it.id == rule.id) it.copy(enabled = enabled) else it
                        }
                        settingsStore.updateTextReplaceRules(updated)
                    },
                    onDelete = {
                        val updated = settingsStore.textReplaceRules.filter { it.id != rule.id }
                        settingsStore.updateTextReplaceRules(updated)
                    }
                )
            }
        }
        
        Spacer(modifier = Modifier.height(100.dp))
    }

    if (showAddDialog || editingRule != null) {
        RuleEditDialog(
            rule = editingRule,
            onDismiss = {
                showAddDialog = false
                editingRule = null
            },
            onSave = { newRule ->
                val current = settingsStore.textReplaceRules
                val updated = if (editingRule != null) {
                    current.map { if (it.id == newRule.id) newRule else it }
                } else {
                    current + newRule
                }
                settingsStore.updateTextReplaceRules(updated)
                showAddDialog = false
                editingRule = null
            }
        )
    }
}

@Composable
private fun RuleItem(
    rule: TextReplaceRule,
    onEdit: (TextReplaceRule) -> Unit,
    onToggle: (Boolean) -> Unit,
    onDelete: () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f),
        onClick = { onEdit(rule) }
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(rule.pattern, style = MaterialTheme.typography.bodyLarge, fontWeight = androidx.compose.ui.text.font.FontWeight.Bold)
                Text("replaces with: ${rule.replacement}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Row(modifier = Modifier.padding(top = 4.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (rule.isRegex) Badge("Regex")
                    if (rule.matchSent) Badge("Incoming")
                    if (rule.matchUnsent) Badge("Outgoing")
                }
            }
            Switch(checked = rule.enabled, onCheckedChange = onToggle)
            IconButton(onClick = onDelete) {
                Icon(Icons.Default.Delete, null, tint = MaterialTheme.colorScheme.error)
            }
        }
    }
}

@Composable
private fun Badge(text: String) {
    Surface(
        color = MaterialTheme.colorScheme.primaryContainer,
        shape = RoundedCornerShape(4.dp)
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onPrimaryContainer
        )
    }
}

@Composable
private fun RuleEditDialog(
    rule: TextReplaceRule?,
    onDismiss: () -> Unit,
    onSave: (TextReplaceRule) -> Unit
) {
    var pattern by remember { mutableStateOf(rule?.pattern ?: "") }
    var replacement by remember { mutableStateOf(rule?.replacement ?: "") }
    var isRegex by remember { mutableStateOf(rule?.isRegex ?: false) }
    var ignoreCase by remember { mutableStateOf(rule?.ignoreCase ?: true) }
    var matchSent by remember { mutableStateOf(rule?.matchSent ?: false) }
    var matchUnsent by remember { mutableStateOf(rule?.matchUnsent ?: true) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(if (rule == null) "Add Rule" else "Edit Rule") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = pattern,
                    onValueChange = { pattern = it },
                    label = { Text("Search Pattern") },
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = replacement,
                    onValueChange = { replacement = it },
                    label = { Text("Replacement Text") },
                    modifier = Modifier.fillMaxWidth()
                )
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(checked = isRegex, onCheckedChange = { isRegex = it })
                    Text("Use Regular Expression", style = MaterialTheme.typography.bodyMedium)
                }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(checked = ignoreCase, onCheckedChange = { ignoreCase = it })
                    Text("Ignore Case", style = MaterialTheme.typography.bodyMedium)
                }
                HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
                Text("Match in:", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(checked = matchUnsent, onCheckedChange = { matchUnsent = it })
                    Text("Outgoing Messages (Before Sending)", style = MaterialTheme.typography.bodyMedium)
                }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(checked = matchSent, onCheckedChange = { matchSent = it })
                    Text("Incoming Messages (Rendering)", style = MaterialTheme.typography.bodyMedium)
                }
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    onSave(
                        TextReplaceRule(
                            id = rule?.id ?: me.lampu.lampcord.shared.utils.Snowflake.nextId(),
                            pattern = pattern,
                            replacement = replacement,
                            isRegex = isRegex,
                            ignoreCase = ignoreCase,
                            enabled = rule?.enabled ?: true,
                            matchSent = matchSent,
                            matchUnsent = matchUnsent
                        )
                    )
                },
                enabled = pattern.isNotBlank()
            ) {
                Text("Save")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        }
    )
}
