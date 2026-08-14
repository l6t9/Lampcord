package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonPrimitive
import me.lampu.lampcord.shared.model.ApplicationCommandOption
import me.lampu.lampcord.shared.state.CommandStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private const val OPTION_SUB_COMMAND = 1
private const val OPTION_SUB_COMMAND_GROUP = 2
private const val OPTION_STRING = 3
private const val OPTION_INTEGER = 4
private const val OPTION_BOOLEAN = 5
private const val OPTION_USER = 6
private const val OPTION_CHANNEL = 7
private const val OPTION_ROLE = 8
private const val OPTION_MENTIONABLE = 9
private const val OPTION_NUMBER = 10
private const val OPTION_ATTACHMENT = 11

fun JsonElement.stringValue(): String? = (this as? JsonPrimitive)?.let { p ->
    if (p.isString) p.content else p.toString()
}

@Composable
fun CommandParameterUI(
    commandStore: CommandStore = koinInject(),
    guildStore: GuildStore = koinInject(),
    memberListStore: MemberListStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val command = commandStore.activeCommand ?: return
    val app = commandStore.availableApplications.find { it.id == command.application_id }

    val options = command.options.orEmpty()
    val groups = options.filter { it.type == OPTION_SUB_COMMAND_GROUP }
    val subs = options.filter { it.type == OPTION_SUB_COMMAND }

    val selectedGroup = commandStore.selectedGroup
    val selectedSub = commandStore.selectedSubCommand

    val leafOptions = when {
        selectedGroup != null -> selectedGroup.options.orEmpty().filter { it.type == OPTION_SUB_COMMAND }
        selectedSub != null -> selectedSub.options.orEmpty()
        else -> options.filter { it.type != OPTION_SUB_COMMAND && it.type != OPTION_SUB_COMMAND_GROUP }
    }

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 4.dp),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
    ) {
        Column(
            modifier = Modifier
                .heightIn(max = 220.dp)
                .verticalScroll(rememberScrollState())
                .padding(12.dp)
        ) {
            // Header: bot icon + command name + description
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                if (app?.icon != null) {
                    AsyncImage(
                        model = "https://cdn.discordapp.com/app-icons/${app.id}/${app.icon}.png?size=64",
                        contentDescription = null,
                        modifier = Modifier.size(32.dp).clip(CircleShape)
                    )
                } else {
                    Box(
                        modifier = Modifier.size(32.dp).background(MaterialTheme.colorScheme.surfaceContainerHighest, CircleShape),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = Icons.Filled.Terminal,
                            contentDescription = null,
                            modifier = Modifier.size(20.dp),
                            tint = MaterialTheme.colorScheme.primary
                        )
                    }
                }
                Spacer(Modifier.width(10.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "/${command.name}",
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Bold
                    )
                    val headerSub = app?.name ?: command.description?.take(60)
                    if (headerSub != null) {
                        Text(
                            text = headerSub,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1
                        )
                    }
                }
                IconButton(
                    onClick = {
                        commandStore.activeCommand = null
                        commandStore.commandOptions.clear()
                        commandStore.resetSubCommand()
                    },
                    modifier = Modifier.size(28.dp)
                ) {
                    Icon(
                        imageVector = Icons.Filled.Close,
                        contentDescription = "Cancel",
                        modifier = Modifier.size(16.dp)
                    )
                }
            }

            if (app == null && command.description != null) {
                Text(
                    text = command.description,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }

            Spacer(Modifier.height(8.dp))

            // Sub-command / sub-command group selector
            val visibleSubs = when {
                selectedGroup != null -> selectedGroup.options.orEmpty()
                groups.isNotEmpty() -> emptyList()
                else -> subs
            }
            if (groups.isNotEmpty()) {
                OptionChipRow(
                    labels = groups.map { it.name },
                    selected = selectedGroup?.name,
                    onSelect = { name ->
                        commandStore.commandOptions.clear()
                        val g = groups.first { it.name == name }
                        commandStore.selectedGroup = g
                        commandStore.selectedSubCommand = null
                    }
                )
            }
            if (visibleSubs.isNotEmpty() && (selectedGroup != null || groups.isEmpty())) {
                OptionChipRow(
                    labels = visibleSubs.map { it.name },
                    selected = selectedSub?.name,
                    onSelect = { name ->
                        commandStore.commandOptions.clear()
                        commandStore.selectedSubCommand = visibleSubs.first { it.name == name }
                    }
                )
            }

            leafOptions.forEach { option ->
                OptionField(
                    option = option,
                    commandStore = commandStore,
                    guildStore = guildStore,
                    memberListStore = memberListStore,
                    navigationStore = navigationStore
                )
            }
        }
    }
}

@Composable
private fun OptionChipRow(
    labels: List<String>,
    selected: String?,
    onSelect: (String) -> Unit
) {
    if (labels.isEmpty()) return
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        labels.forEach { label ->
            val isSelected = label == selected
            Surface(
                onClick = { onSelect(label) },
                shape = RoundedCornerShape(16.dp),
                color = if (isSelected) MaterialTheme.colorScheme.primary.copy(alpha = 0.2f)
                else MaterialTheme.colorScheme.surface.copy(alpha = 0.6f),
                border = androidx.compose.foundation.BorderStroke(
                    1.dp,
                    if (isSelected) MaterialTheme.colorScheme.primary.copy(alpha = 0.5f)
                    else MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f)
                )
            ) {
                Text(
                    text = label,
                    style = MaterialTheme.typography.labelMedium,
                    color = if (isSelected) MaterialTheme.colorScheme.primary
                    else MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                    maxLines = 1
                )
            }
        }
    }
}

@Composable
private fun OptionField(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    guildStore: GuildStore,
    memberListStore: MemberListStore,
    navigationStore: NavigationStore
) {
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = option.name,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface
            )
            if (option.required) {
                Text("*", color = MaterialTheme.colorScheme.error)
            }
            Spacer(Modifier.width(8.dp))
            Text(
                text = option.description,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1
            )
        }

        Spacer(Modifier.height(4.dp))

        when (option.type) {
            OPTION_STRING -> TextOptionField(option, commandStore, null, KeyboardType.Text)
            OPTION_INTEGER -> TextOptionField(option, commandStore, validate = { it.toLongOrNull() != null }, KeyboardType.Number)
            OPTION_NUMBER -> TextOptionField(option, commandStore, validate = { it.toDoubleOrNull() != null }, KeyboardType.Decimal)
            OPTION_BOOLEAN -> BooleanOptionField(option, commandStore)
            OPTION_USER, OPTION_MENTIONABLE -> MentionOptionField(
                option = option,
                commandStore = commandStore,
                memberListStore = memberListStore,
                navigationStore = navigationStore,
                prefix = "@"
            )
            OPTION_CHANNEL -> MentionOptionField(
                option = option,
                commandStore = commandStore,
                guildStore = guildStore,
                navigationStore = navigationStore,
                prefix = "#"
            )
            OPTION_ROLE -> MentionOptionField(
                option = option,
                commandStore = commandStore,
                navigationStore = navigationStore,
                prefix = "@"
            )
            OPTION_ATTACHMENT -> AttachmentOptionField(option, commandStore)
            else -> TextOptionField(option, commandStore, null, KeyboardType.Text)
        }
    }
}

@Composable
private fun TextOptionField(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    validate: ((String) -> Boolean)?,
    keyboardType: KeyboardType
) {
    var text by remember(option.name) { mutableStateOf(commandStore.commandOptions[option.name]?.stringValue() ?: "") }
    val hasChoices = option.choices?.isNotEmpty() == true

    if (hasChoices) {
        ChoiceOptionField(option, commandStore)
        return
    }

    BasicTextField(
        value = text,
        onValueChange = { new ->
            text = new
            when {
                new.isEmpty() -> commandStore.commandOptions.remove(option.name)
                validate?.invoke(new) != false -> commandStore.commandOptions[option.name] =
                    when (option.type) {
                        OPTION_INTEGER -> JsonPrimitive(new.toLongOrNull() ?: new.toLong())
                        OPTION_NUMBER -> JsonPrimitive(new.toDoubleOrNull() ?: 0.0)
                        else -> JsonPrimitive(new)
                    }
            }
        },
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
        keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
        modifier = Modifier
            .fillMaxWidth()
            .height(34.dp)
            .clip(RoundedCornerShape(4.dp))
            .background(MaterialTheme.colorScheme.surface)
            .padding(horizontal = 8.dp),
        decorationBox = { inner ->
            if (text.isEmpty()) {
                Text(
                    text = option.description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                    maxLines = 1
                )
            }
            inner()
        }
    )
}

@Composable
private fun ChoiceOptionField(
    option: ApplicationCommandOption,
    commandStore: CommandStore
) {
    var expanded by remember(option.name) { mutableStateOf(false) }
    val choices = option.choices.orEmpty()
    val selected = commandStore.commandOptions[option.name]
    val selectedName = choices.firstOrNull { it.value == selected }?.name

    Box {
        Surface(
            onClick = { expanded = true },
            shape = RoundedCornerShape(4.dp),
            color = MaterialTheme.colorScheme.surface,
            modifier = Modifier.fillMaxWidth()
        ) {
            Text(
                text = selectedName ?: "Select ${option.name}...",
                style = MaterialTheme.typography.bodyMedium,
                color = if (selectedName != null) MaterialTheme.colorScheme.onSurface
                else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp),
                maxLines = 1
            )
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            choices.forEach { choice ->
                DropdownMenuItem(
                    text = { Text(choice.name) },
                    onClick = {
                        commandStore.commandOptions[option.name] = choice.value
                        expanded = false
                    }
                )
            }
        }
    }
}

@Composable
private fun BooleanOptionField(
    option: ApplicationCommandOption,
    commandStore: CommandStore
) {
    var checked by remember(option.name) {
        mutableStateOf(commandStore.commandOptions[option.name]?.toString()?.contains("true") == true)
    }
    Row(verticalAlignment = Alignment.CenterVertically) {
        Switch(
            checked = checked,
            onCheckedChange = {
                checked = it
                commandStore.commandOptions[option.name] = JsonPrimitive(it)
            }
        )
        Spacer(Modifier.width(8.dp))
        Text(
            text = checked.toString(),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface
        )
    }
}

@Composable
private fun MentionOptionField(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    memberListStore: MemberListStore? = null,
    guildStore: GuildStore? = null,
    navigationStore: NavigationStore,
    prefix: String
) {
    val storedId = commandStore.commandOptions[option.name]?.stringValue()
    var selected by remember(option.name) { mutableStateOf<String?>(storedId) }
    var text by remember(option.name) { mutableStateOf("") }
    var showSuggestions by remember(option.name) { mutableStateOf(true) }

    val guildId = navigationStore.selectedGuild?.id
    val candidates = remember(option.name, guildId, memberListStore, guildStore) {
        buildCandidateList(option, memberListStore, guildStore, navigationStore)
    }

    // Restore the display name from the stored id.
    if (storedId != null && text.isEmpty() && selected == storedId) {
        text = candidates.firstOrNull { it.second == storedId }?.first ?: ""
    }

    val filtered = if (showSuggestions && text.isNotBlank()) {
        candidates.filter { it.first.contains(text, ignoreCase = true) }.take(6)
    } else emptyList()

    Column(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(34.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surface)
                .padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = prefix,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            BasicTextField(
                value = text,
                onValueChange = { new ->
                    text = new
                    showSuggestions = true
                    if (new != candidates.firstOrNull { it.second == selected }?.first) {
                        selected = null
                        commandStore.commandOptions.remove(option.name)
                    }
                },
                singleLine = true,
                textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
                modifier = Modifier.weight(1f),
                decorationBox = { inner ->
                    if (text.isEmpty()) {
                        Text(
                            text = option.description,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                            maxLines = 1
                        )
                    }
                    inner()
                }
            )
            if (selected != null) {
                IconButton(
                    onClick = {
                        selected = null
                        text = ""
                        commandStore.commandOptions.remove(option.name)
                    },
                    modifier = Modifier.size(22.dp)
                ) {
                    Icon(Icons.Filled.Close, null, modifier = Modifier.size(14.dp))
                }
            }
        }

        if (filtered.isNotEmpty()) {
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 2.dp),
                shape = RoundedCornerShape(4.dp),
                color = MaterialTheme.colorScheme.surfaceContainerHigh
            ) {
                Column {
                    filtered.forEachIndexed { index, (name, id) ->
                        if (index > 0) HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.2f))
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable {
                                    text = name
                                    selected = id
                                    showSuggestions = false
                                    commandStore.commandOptions[option.name] = JsonPrimitive(id)
                                }
                                .padding(horizontal = 8.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(prefix, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
                            Spacer(Modifier.width(2.dp))
                            Text(name, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurface, maxLines = 1)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun AttachmentOptionField(
    option: ApplicationCommandOption,
    commandStore: CommandStore
) {
    var text by remember(option.name) { mutableStateOf(commandStore.commandOptions[option.name]?.stringValue() ?: "") }
    BasicTextField(
        value = text,
        onValueChange = {
            text = it
            if (it.isEmpty()) commandStore.commandOptions.remove(option.name)
            else commandStore.commandOptions[option.name] = JsonPrimitive(it)
        },
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
        modifier = Modifier
            .fillMaxWidth()
            .height(34.dp)
            .clip(RoundedCornerShape(4.dp))
            .background(MaterialTheme.colorScheme.surface)
            .padding(horizontal = 8.dp),
        decorationBox = { inner ->
            if (text.isEmpty()) {
                Text(
                    text = option.description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                    maxLines = 1
                )
            }
            inner()
        }
    )
}

private fun buildCandidateList(
    option: ApplicationCommandOption,
    memberListStore: MemberListStore?,
    guildStore: GuildStore?,
    navigationStore: NavigationStore
): List<Pair<String, String>> {
    val guildId = navigationStore.selectedGuild?.id
    return when (option.type) {
        OPTION_USER, OPTION_MENTIONABLE -> {
            memberListStore?.memberListItems
                ?.mapNotNull { it?.member }
                ?.mapNotNull { member ->
                    val user = member.user ?: return@mapNotNull null
                    val name = member.nick ?: user.global_name ?: user.username ?: return@mapNotNull null
                    name to user.id
                }
                ?.distinctBy { it.second }
                ?: emptyList()
        }
        OPTION_CHANNEL -> {
            val allowed = option.channel_types?.toSet()
            guildStore?.allGuildChannels?.value?.values
                ?.filter { it.guild_id == guildId }
                ?.filter { allowed == null || it.type in allowed }
                ?.mapNotNull { c -> c.name?.let { it to c.id } }
                ?: emptyList()
        }
        OPTION_ROLE -> {
            navigationStore.selectedGuild?.roles?.map { it.name to it.id } ?: emptyList()
        }
        else -> emptyList()
    }
}
