package com.example.materialcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.example.materialcord.shared.api.DiscordClient
import com.example.materialcord.shared.gateway.GatewayManager
import com.example.materialcord.shared.model.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.Json

class ChatState(
    private val gatewayManager: GatewayManager,
    private val discordClient: DiscordClient,
    private val json: Json
) {
    var isConnected by mutableStateOf(false)
    val messages = mutableStateListOf<Message>()
    val guilds = mutableStateListOf<Guild>()
    val channels = mutableStateListOf<Channel>()
    
    var currentUser by mutableStateOf<User?>(null)
    var selectedGuild by mutableStateOf<Guild?>(null)
    var selectedChannel by mutableStateOf<Channel?>(null)
    
    private var currentToken: String? = null

    private val scope = CoroutineScope(Dispatchers.Main)

    init {
        scope.launch {
            gatewayManager.events.collectLatest { payload ->
                when (payload.t) {
                    "READY" -> {
                        isConnected = true
                        payload.d?.let { data ->
                            try {
                                val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                                currentUser = ready.user
                                guilds.clear()
                                guilds.addAll(ready.guilds)
                            } catch (e: Exception) {
                                println("Error decoding READY: ${e.message}")
                            }
                        }
                    }
                    "GUILD_CREATE" -> {
                        payload.d?.let { data ->
                            try {
                                val guild = json.decodeFromJsonElement<Guild>(data)
                                if (guilds.none { it.id == guild.id }) {
                                    guilds.add(guild)
                                } else {
                                    // Update existing guild info if it was just an ID before
                                    val index = guilds.indexOfFirst { it.id == guild.id }
                                    guilds[index] = guild
                                }
                            } catch (e: Exception) {
                                println("Error decoding GUILD_CREATE: ${e.message}")
                            }
                        }
                    }
                    "MESSAGE_CREATE" -> {
                        payload.d?.let { data ->
                            try {
                                val message = json.decodeFromJsonElement<Message>(data)
                                if (selectedChannel?.id == message.channel_id) {
                                    if (messages.none { it.id == message.id }) {
                                        messages.add(0, message)
                                    }
                                }
                            } catch (e: Exception) {
                                println("Error decoding message: ${e.message}")
                            }
                        }
                    }
                }
            }
        }
    }

    fun connect(token: String) {
        currentToken = token
        discordClient.setToken(token)
        scope.launch {
            gatewayManager.connect(token)
        }
    }

    fun selectGuild(guild: Guild) {
        selectedGuild = guild
        selectedChannel = null
        channels.clear()
        messages.clear()
        
        scope.launch {
            val guildChannels = discordClient.getGuildChannels(guild.id)
            channels.addAll(guildChannels.filter { it.type == 0 || it.type == 5 }.sortedBy { it.position })
        }
    }

    fun selectChannel(channel: Channel) {
        selectedChannel = channel
        messages.clear()
        
        scope.launch {
            val channelMessages = discordClient.getChannelMessages(channel.id)
            messages.addAll(channelMessages)
        }
    }

    fun sendMessage(content: String) {
        val channelId = selectedChannel?.id ?: return
        scope.launch {
            discordClient.sendMessage(channelId, content)
        }
    }
}
