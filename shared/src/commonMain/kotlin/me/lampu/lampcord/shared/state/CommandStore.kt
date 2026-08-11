package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.serialization.json.JsonElement
import kotlin.random.Random

class CommandStore(
    private val discordClient: DiscordClient,
    private val gatewayManager: GatewayManager,
    private val scope: CoroutineScope
) {
    val availableCommands = mutableStateListOf<ApplicationCommand>()
    val availableApplications = mutableStateListOf<Application>()

    var activeCommand by mutableStateOf<ApplicationCommand?>(null)
    val commandOptions = mutableStateMapOf<String, JsonElement>()

    fun setCommands(commands: List<ApplicationCommand>, applications: List<Application>) {
        availableCommands.clear()
        availableCommands.addAll(commands)
        availableApplications.clear()
        availableApplications.addAll(applications)
    }

    fun sendInteraction(command: ApplicationCommand, guildId: String?, channelId: String, options: List<InteractionOption>? = null) {
        scope.launch {
            discordClient.sendInteraction(
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
    }
}
