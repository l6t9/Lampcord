package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.*
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonElement
import kotlin.random.Random

class CommandStore(
    private val guildApi: GuildApi,
    private val gatewayManager: GatewayManager,
    private val scope: CoroutineScope
) {
    val availableCommands = mutableStateListOf<ApplicationCommand>()
    val availableApplications = mutableStateListOf<Application>()

    var activeCommand by mutableStateOf<ApplicationCommand?>(null)
    val commandOptions = mutableStateMapOf<String, JsonElement>()

    var selectedGroup by mutableStateOf<ApplicationCommandOption?>(null)
    var selectedSubCommand by mutableStateOf<ApplicationCommandOption?>(null)

    fun setCommands(commands: List<ApplicationCommand>, applications: List<Application>) {
        availableCommands.clear()
        availableCommands.addAll(commands)
        availableApplications.clear()
        availableApplications.addAll(applications)
    }

    fun resetSubCommand() {
        selectedGroup = null
        selectedSubCommand = null
    }

    private fun leafOptions(): List<ApplicationCommandOption> {
        val cmd = activeCommand ?: return emptyList()
        val opts = selectedGroup?.options
            ?: selectedSubCommand?.options
            ?: cmd.options.orEmpty()
        return opts.filter { it.type != 1 && it.type != 2 }
    }

    fun hasSubcommands(): Boolean {
        val cmd = activeCommand ?: return false
        return cmd.options.orEmpty().any { it.type == 1 || it.type == 2 }
    }

    fun isCommandValid(): Boolean {
        val cmd = activeCommand ?: return false
        val hasSubs = cmd.options.orEmpty().any { it.type == 1 || it.type == 2 }
        if (hasSubs && selectedSubCommand == null) return false
        return leafOptions().all { opt ->
            !opt.required || commandOptions[opt.name] != null
        }
    }

    fun buildInteractionOptions(): List<InteractionOption>? {
        val cmd = activeCommand ?: return null
        val leaf = leafOptions().mapNotNull { opt ->
            val value = commandOptions[opt.name] ?: return@mapNotNull null
            InteractionOption(type = opt.type, name = opt.name, value = value)
        }
        return when {
            selectedGroup != null && selectedSubCommand != null -> listOf(
                InteractionOption(type = 2, name = selectedGroup!!.name, options = listOf(
                    InteractionOption(type = 1, name = selectedSubCommand!!.name, options = leaf)
                ))
            )
            selectedSubCommand != null -> listOf(
                InteractionOption(type = 1, name = selectedSubCommand!!.name, options = leaf)
            )
            else -> leaf.ifEmpty { null }
        }
    }

    fun sendInteraction(command: ApplicationCommand, guildId: String?, channelId: String, options: List<InteractionOption>? = null) {
        scope.launch {
            guildApi.sendInteraction(
                InteractionRequest(
                    type = 2,
                    application_id = command.application_id,
                    guild_id = guildId,
                    channel_id = channelId,
                    session_id = gatewayManager.sessionId ?: "",
                    data = InteractionData(
                        id = command.id,
                        name = command.name,
                        type = command.type,
                        version = command.version,
                        options = options
                    ),
                    nonce = "${getCurrentTimeMillis()}${Random.nextInt(1000, 9999)}"
                )
            )
        }
    }

    fun clear() {
        availableCommands.clear()
        availableApplications.clear()
        resetSubCommand()
    }
}
