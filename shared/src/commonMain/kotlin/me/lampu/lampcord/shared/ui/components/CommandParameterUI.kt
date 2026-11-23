package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonPrimitive
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.cancel
import me.lampu.lampcord.shared.model.ApplicationCommandOption
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.CommandStore
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.jetbrains.compose.resources.stringResource
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

fun JsonElement.stringValue(): String? = (this as? JsonPrimitive)?.let { primitive ->
    if (primitive.isString) primitive.content else primitive.toString()
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
        selectedGroup != null && selectedSub != null -> selectedSub.options.orEmpty()
        selectedGroup != null -> selectedGroup.options.orEmpty().filter { it.type == OPTION_SUB_COMMAND }
        selectedSub != null -> selectedSub.options.orEmpty()
        groups.isNotEmpty() -> emptyList()
        else -> options.filter { it.type != OPTION_SUB_COMMAND && it.type != OPTION_SUB_COMMAND_GROUP }
    }

    val pillOptions = when {
        selectedGroup != null && selectedSub == null -> selectedGroup.options.orEmpty().filter { it.type == OPTION_SUB_COMMAND }
        selectedGroup == null && groups.isNotEmpty() -> groups
        selectedSub == null && subs.isNotEmpty() -> subs
        else -> leafOptions
    }

    val activeOption = pillOptions.firstOrNull { it.name == commandStore.activeOptionName }
    val reduceMotion = Settings.shared.reduceMotion
    val scrollState = rememberLazyListState()

    LaunchedEffect(commandStore.activeOptionName, pillOptions.size) {
        val name = commandStore.activeOptionName ?: return@LaunchedEffect
        val index = pillOptions.indexOfFirst { it.name == name }
        if (index >= 0) {
            // Two leading items: application icon and command name.
            val row = index + 2
            if (reduceMotion) scrollState.scrollToItem(row) else scrollState.animateScrollToItem(row)
        }
    }

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 12.dp, vertical = 4.dp),
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        border = androidx.compose.foundation.BorderStroke(
            1.dp,
            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.45f)
        ),
        tonalElevation = 2.dp
    ) {
        Column {
            LazyRow(
                state = scrollState,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(48.dp),
                contentPadding = PaddingValues(horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                item {
                    if (app?.icon != null) {
                        AsyncImage(
                            model = "https://cdn.discordapp.com/app-icons/${app.id}/${app.icon}.png?size=32",
                            contentDescription = null,
                            modifier = Modifier.size(24.dp).clip(CircleShape)
                        )
                    } else {
                        Box(
                            modifier = Modifier
                                .size(24.dp)
                                .background(
                                    MaterialTheme.colorScheme.primaryContainer,
                                    CircleShape
                                ),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                imageVector = Icons.Filled.Terminal,
                                contentDescription = null,
                                modifier = Modifier.size(15.dp),
                                tint = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        }
                    }
                }
                item {
                    Text(
                        text = "/${command.name}",
                        style = MaterialTheme.typography.bodyMedium.copy(fontSize = 14.sp),
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1
                    )
                }
                itemsIndexed(pillOptions, key = { _, option -> option.name }) { _, option ->
                    CommandOptionPill(
                        option = option,
                        commandStore = commandStore,
                        isActive = commandStore.activeOptionName == option.name,
                        onClick = {
                            commandStore.activeOptionName =
                                if (commandStore.activeOptionName == option.name) null else option.name
                        }
                    )
                }
                item {
                    IconButton(
                        onClick = { commandStore.clearActiveCommand() },
                        modifier = Modifier.size(28.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Filled.Close,
                            contentDescription = stringResource(Res.string.cancel),
                            modifier = Modifier.size(16.dp)
                        )
                    }
                }
            }

            if (activeOption != null) {
                HorizontalDivider(
                    color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f)
                )
                CommandOptionEditor(
                    option = activeOption,
                    commandStore = commandStore,
                    guildStore = guildStore,
                    memberListStore = memberListStore,
                    navigationStore = navigationStore,
                    leafOptions = leafOptions,
                    onDone = { commandStore.activeOptionName = null }
                )
            }
        }
    }
}

@Composable
private fun CommandOptionPill(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    isActive: Boolean,
    onClick: () -> Unit
) {
    val complete = isCommandOptionComplete(option, commandStore)
    val invalid = option.required && !complete && commandStore.activeOptionName != null && !isActive
    val background = when {
        isActive -> MaterialTheme.colorScheme.primary
        complete -> MaterialTheme.colorScheme.primary.copy(alpha = 0.18f)
        invalid -> MaterialTheme.colorScheme.error.copy(alpha = 0.14f)
        else -> MaterialTheme.colorScheme.surfaceContainerHighest
    }
    val foreground = when {
        isActive -> MaterialTheme.colorScheme.onPrimary
        invalid -> MaterialTheme.colorScheme.error
        complete -> MaterialTheme.colorScheme.onSurfaceVariant
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }

    Box(
        modifier = Modifier
            .height(30.dp)
            .clip(RoundedCornerShape(15.dp))
            .background(background)
            .clickable(interactionSource = null, indication = null, onClick = onClick)
            .padding(horizontal = 10.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = if (option.required) "${option.name}*" else option.name,
            style = MaterialTheme.typography.labelMedium.copy(fontSize = 13.sp),
            color = foreground,
            maxLines = 1
        )
    }
}

private fun isCommandOptionComplete(
    option: ApplicationCommandOption,
    commandStore: CommandStore
): Boolean = when (option.type) {
    OPTION_SUB_COMMAND_GROUP -> commandStore.selectedGroup?.name == option.name
    OPTION_SUB_COMMAND -> commandStore.selectedSubCommand?.name == option.name
    else -> commandStore.commandOptions[option.name] != null
}

@Composable
private fun CommandOptionEditor(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    guildStore: GuildStore,
    memberListStore: MemberListStore,
    navigationStore: NavigationStore,
    leafOptions: List<ApplicationCommandOption>,
    onDone: () -> Unit
) {
    val scrollState = rememberScrollState()

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(max = 200.dp)
            .verticalScroll(scrollState)
            .padding(horizontal = 12.dp, vertical = 8.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = option.name,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface
            )
            if (option.required) {
                Spacer(Modifier.width(3.dp))
                Text("*", color = MaterialTheme.colorScheme.error)
            }
        }
        Text(
            text = option.description,
            style = MaterialTheme.typography.bodySmall.copy(fontSize = 14.sp),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = 2.dp)
        )
        Spacer(Modifier.height(8.dp))

        when (option.type) {
            OPTION_SUB_COMMAND_GROUP, OPTION_SUB_COMMAND -> {
                option.options.orEmpty().forEach { nested ->
                    CommandSelectionRow(
                        title = nested.name,
                        subtitle = nested.description,
                        selected = when (option.type) {
                            OPTION_SUB_COMMAND_GROUP -> commandStore.selectedGroup?.name == nested.name
                            else -> commandStore.selectedSubCommand?.name == nested.name
                        }
                    ) {
                        commandStore.commandOptions.clear()
                        if (option.type == OPTION_SUB_COMMAND_GROUP) {
                            commandStore.selectedGroup = nested
                            commandStore.selectedSubCommand = null
                        } else {
                            commandStore.selectedSubCommand = nested
                        }
                        onDone()
                    }
                }
            }
            OPTION_BOOLEAN -> {
                var checked by remember(option.name) {
                    mutableStateOf(commandStore.commandOptions[option.name]?.toString()?.contains("true") == true)
                }
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(48.dp)
                        .clickable(interactionSource = null, indication = null) {
                            checked = !checked
                            commandStore.commandOptions[option.name] = JsonPrimitive(checked)
                            advanceOption(commandStore, leafOptions, option.name)
                            onDone()
                        }
                        .padding(horizontal = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = checked.toString(),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.weight(1f)
                    )
                    Switch(checked = checked, onCheckedChange = null)
                }
            }
            else -> {
                val choices = option.choices.orEmpty()
                if (choices.isNotEmpty()) {
                    choices.forEach { choice ->
                        val selected = commandStore.commandOptions[option.name] == choice.value
                        CommandSelectionRow(
                            title = choice.name,
                            subtitle = null,
                            selected = selected
                        ) {
                            commandStore.commandOptions[option.name] = choice.value
                            advanceOption(commandStore, leafOptions, option.name)
                            onDone()
                        }
                    }
                } else if (isMentionOption(option.type)) {
                    MentionOptionValue(
                        option = option,
                        commandStore = commandStore,
                        memberListStore = memberListStore,
                        guildStore = guildStore,
                        navigationStore = navigationStore,
                        leafOptions = leafOptions
                    )
                } else {
                    TextOptionValue(
                        option = option,
                        commandStore = commandStore,
                        leafOptions = leafOptions
                    )
                }
            }
        }
    }
}

private fun isMentionOption(type: Int): Boolean =
    type == OPTION_USER || type == OPTION_CHANNEL || type == OPTION_ROLE || type == OPTION_MENTIONABLE

private fun advanceOption(
    commandStore: CommandStore,
    leafOptions: List<ApplicationCommandOption>,
    completedName: String
) {
    val next = leafOptions.firstOrNull {
        it.name != completedName && commandStore.commandOptions[it.name] == null
    }
    commandStore.activeOptionName = next?.name
}

@Composable
private fun CommandSelectionRow(
    title: String,
    subtitle: String?,
    selected: Boolean,
    onClick: () -> Unit
) {
    val background = if (selected) {
        MaterialTheme.colorScheme.primary.copy(alpha = 0.14f)
    } else {
        Color.Transparent
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .background(background, RoundedCornerShape(8.dp))
            .clickable(interactionSource = null, indication = null, onClick = onClick)
            .padding(horizontal = 8.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = if (selected) "✓" else "",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.width(20.dp)
        )
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            if (subtitle != null) {
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 14.sp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

@Composable
private fun TextOptionValue(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    leafOptions: List<ApplicationCommandOption>
) {
    var text by remember(option.name) {
        mutableStateOf(commandStore.commandOptions[option.name]?.stringValue() ?: "")
    }
    val keyboardType = when (option.type) {
        OPTION_INTEGER -> KeyboardType.Number
        OPTION_NUMBER -> KeyboardType.Decimal
        else -> KeyboardType.Text
    }
    val valid: (String) -> Boolean = { value ->
        when (option.type) {
            OPTION_INTEGER -> value.toLongOrNull() != null
            OPTION_NUMBER -> value.toDoubleOrNull() != null
            else -> true
        }
    }

    BasicTextField(
        value = text,
        onValueChange = { value ->
            text = value
            if (value.isEmpty()) {
                commandStore.commandOptions.remove(option.name)
            } else if (valid(value)) {
                commandStore.commandOptions[option.name] = when (option.type) {
                    OPTION_INTEGER -> JsonPrimitive(value.toLongOrNull() ?: 0L)
                    OPTION_NUMBER -> JsonPrimitive(value.toDoubleOrNull() ?: 0.0)
                    else -> JsonPrimitive(value)
                }
            }
        },
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
        keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
        modifier = Modifier
            .fillMaxWidth()
            .height(40.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            .padding(horizontal = 10.dp),
        decorationBox = { inner ->
            Box(contentAlignment = Alignment.CenterStart) {
                if (text.isEmpty()) {
                    Text(
                        text = option.description,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                inner()
            }
        }
    )
    Spacer(Modifier.height(8.dp))
    if (commandStore.commandOptions[option.name] != null) {
        CommandSelectionRow(
            title = "Use ${option.name}",
            subtitle = null,
            selected = true
        ) {
            advanceOption(commandStore, leafOptions, option.name)
        }
    }
}

@Composable
private fun MentionOptionValue(
    option: ApplicationCommandOption,
    commandStore: CommandStore,
    memberListStore: MemberListStore,
    guildStore: GuildStore,
    navigationStore: NavigationStore,
    leafOptions: List<ApplicationCommandOption>
) {
    val storedId = commandStore.commandOptions[option.name]?.stringValue()
    var text by remember(option.name) { mutableStateOf("") }
    val prefix = if (option.type == OPTION_CHANNEL) "#" else "@"
    val candidates = remember(option.name, navigationStore.selectedGuild?.id) {
        buildCandidateList(option, memberListStore, guildStore, navigationStore)
    }
    val filtered = if (text.isBlank()) {
        candidates.take(6)
    } else {
        candidates.filter { it.first.contains(text, ignoreCase = true) }.take(6)
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(40.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            .padding(horizontal = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(prefix, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.primary)
        Spacer(Modifier.width(4.dp))
        BasicTextField(
            value = if (storedId != null && text.isEmpty()) {
                candidates.firstOrNull { it.second == storedId }?.first ?: text
            } else {
                text
            },
            onValueChange = { value ->
                text = value
                commandStore.commandOptions.remove(option.name)
            },
            singleLine = true,
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
            modifier = Modifier.weight(1f),
            decorationBox = { inner ->
                Box(contentAlignment = Alignment.CenterStart) {
                    if (text.isEmpty() && storedId == null) {
                        Text(
                            text = option.description,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    inner()
                }
            }
        )
    }

    Spacer(Modifier.height(4.dp))
    filtered.forEach { (name, id) ->
        CommandSelectionRow(
            title = "$prefix$name",
            subtitle = null,
            selected = storedId == id
        ) {
            commandStore.commandOptions[option.name] = JsonPrimitive(id)
            advanceOption(commandStore, leafOptions, option.name)
        }
    }
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
                ?.mapNotNull { channel -> channel.name?.let { it to channel.id } }
                ?: emptyList()
        }
        OPTION_ROLE -> {
            navigationStore.selectedGuild?.roles?.map { it.name to it.id } ?: emptyList()
        }
        else -> emptyList()
    }
}
