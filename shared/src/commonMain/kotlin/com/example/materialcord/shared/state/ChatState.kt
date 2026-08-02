package com.example.materialcord.shared.state

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.example.materialcord.shared.api.DiscordClient
import com.example.materialcord.shared.gateway.GatewayManager
import com.example.materialcord.shared.model.*
import com.example.materialcord.shared.settings.Settings
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.encodeToJsonElement
import kotlinx.serialization.json.Json

class ChatState(
    private val gatewayManager: GatewayManager,
    private val discordClient: DiscordClient,
    private val json: Json
) {
    var isConnected by mutableStateOf(false)
    var isConnecting by mutableStateOf(false)
    val messages = mutableStateListOf<Message>()
    val guilds = mutableStateListOf<Guild>()
    val channels = mutableStateListOf<Channel>()
    val privateChannels = mutableStateListOf<Channel>()
    val forumThreads = mutableStateListOf<Channel>()
    val memberListItems = mutableStateListOf<MemberListListItem?>()
    
    var currentUser by mutableStateOf<User?>(null)
    var selectedGuild by mutableStateOf<Guild?>(null)
    var selectedChannel by mutableStateOf<Channel?>(null)
    var selectedThread by mutableStateOf<Channel?>(null)
    var selectedUser by mutableStateOf<User?>(null)
    var selectedProfile by mutableStateOf<UserProfile?>(null)
    
    var userSettings by mutableStateOf<UserSettings?>(null)
    
    private var currentToken: String? = null
    private var currentFingerprint: String? = null

    private val scope = CoroutineScope(Dispatchers.Main)

    init {
        val savedToken = Settings.shared.discordToken
        if (savedToken.isNotBlank()) {
            connect(savedToken)
        }

        scope.launch {
            gatewayManager.events.collectLatest { payload ->
                when (payload.t) {
                    "READY" -> {
                        isConnected = true
                        isConnecting = false
                        payload.d?.let { data ->
                            try {
                                val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                                currentUser = ready.user
                                userSettings = ready.user_settings
                                
                                guilds.clear()
                                guilds.addAll(ready.guilds)
                                
                                privateChannels.clear()
                                privateChannels.addAll(ready.private_channels.sortedByDescending { it.last_message_id })
                                
                                // Apply ordering if settings exist
                                applyGuildOrdering()
                            } catch (e: Exception) {
                                println("Error decoding READY: ${e.message}")
                                e.printStackTrace()
                            }
                        }
                    }
                    "GUILD_CREATE" -> {
                        payload.d?.let { data ->
                            try {
                                val guild = json.decodeFromJsonElement<Guild>(data)
                                val index = guilds.indexOfFirst { it.id == guild.id }
                                if (index == -1) {
                                    guilds.add(guild)
                                } else {
                                    guilds[index] = guild
                                }
                                applyGuildOrdering()
                            } catch (e: Exception) {
                                println("Error decoding GUILD_CREATE: ${e.message}")
                            }
                        }
                    }
                    "MESSAGE_CREATE" -> {
                        payload.d?.let { data ->
                            try {
                                val message = json.decodeFromJsonElement<Message>(data)
                                
                                // Update DM order if it's a DM
                                val dmIndex = privateChannels.indexOfFirst { it.id == message.channel_id }
                                if (dmIndex != -1) {
                                    val dm = privateChannels.removeAt(dmIndex)
                                    privateChannels.add(0, dm.copy(last_message_id = message.id))
                                }

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
                    "GUILD_MEMBER_LIST_UPDATE" -> {
                        payload.d?.let { data ->
                            try {
                                val update = json.decodeFromJsonElement<MemberListUpdate>(data)
                                if (update.guild_id == selectedGuild?.id) {
                                    handleMemberListUpdate(update)
                                }
                            } catch (e: Exception) {
                                println("Error decoding GUILD_MEMBER_LIST_UPDATE: ${e.message}")
                            }
                        }
                    }
                }
            }
        }
    }

    private fun handleMemberListUpdate(update: MemberListUpdate) {
        // Bulk resize if needed to reduce multiple list updates
        update.member_count?.let { count ->
            if (memberListItems.size != count) {
                if (count > memberListItems.size) {
                    val placeholders = List(count - memberListItems.size) { null }
                    memberListItems.addAll(placeholders)
                } else {
                    repeat(memberListItems.size - count) {
                        memberListItems.removeAt(memberListItems.size - 1)
                    }
                }
            }
        }

        for (op in update.ops) {
            when (op.op) {
                "SYNC" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val items = op.items ?: emptyList()
                    for (i in items.indices) {
                        val index = start + i
                        if (index < memberListItems.size) {
                            memberListItems[index] = items[i]
                        }
                    }
                }
                "INSERT" -> {
                    val index = op.index ?: return
                    val item = op.item ?: return
                    memberListItems.add(index.coerceIn(0, memberListItems.size), item)
                }
                "UPDATE" -> {
                    val index = op.index ?: return
                    val item = op.item ?: return
                    if (index < memberListItems.size) {
                        memberListItems[index] = item
                    }
                }
                "DELETE" -> {
                    val index = op.index ?: return
                    if (index < memberListItems.size) {
                        memberListItems.removeAt(index)
                    }
                }
                "INVALIDATE" -> {
                    val range = op.range ?: continue
                    val start = range[0]
                    val end = range[1]
                    for (i in start..end) {
                        if (i < memberListItems.size) {
                            memberListItems[i] = null
                        }
                    }
                }
            }
        }
    }

    private fun applyGuildOrdering() {
        val order = userSettings?.guild_positions ?: return
        if (order.isEmpty()) return
        
        val sorted = guilds.sortedBy { guild ->
            val pos = order.indexOf(guild.id)
            if (pos == -1) Int.MAX_VALUE else pos
        }
        
        guilds.clear()
        guilds.addAll(sorted)
    }

    fun connect(token: String) {
        isConnecting = true
        currentToken = token
        Settings.shared.discordToken = token
        discordClient.setToken(token)
        scope.launch {
            gatewayManager.connect(token)
        }
    }

    suspend fun login(login: String, password: String): LoginResponse? {
        val fingerprint = discordClient.getFingerprint() ?: return null
        currentFingerprint = fingerprint
        val response = discordClient.login(LoginRequest(login, password), fingerprint)
        
        if (response?.token != null) {
            connect(response.token)
        }
        
        return response
    }

    suspend fun verifyMFA(code: String, ticket: String, type: String = "totp"): Boolean {
        val fingerprint = currentFingerprint ?: discordClient.getFingerprint() ?: return false
        val response = discordClient.loginMFA(MFALoginRequest(code, ticket), fingerprint, type)
        
        if (response?.token != null) {
            connect(response.token)
            return true
        }
        
        return false
    }

    fun selectGuild(guild: Guild) {
        selectedGuild = guild
        selectedChannel = null
        selectedThread = null
        channels.clear()
        messages.clear()
        forumThreads.clear()
        memberListItems.clear()
        
        scope.launch {
            val guildChannels = discordClient.getGuildChannels(guild.id)
            channels.addAll(guildChannels.filter { it.type == 0 || it.type == 5 || it.type == 4 || it.type == 15 }.sortedBy { it.position })
        }
    }

    fun selectChannel(channel: Channel) {
        selectedChannel = channel
        selectedThread = null
        messages.clear()
        forumThreads.clear()
        
        scope.launch {
            if (channel.type == 15) {
                val threads = discordClient.getActiveThreads(channel.id)
                threads?.threads?.let {
                    forumThreads.addAll(it)
                }
            } else {
                val channelMessages = discordClient.getChannelMessages(channel.id)
                messages.addAll(channelMessages)
            }
            
            // Subscribe to member list
            val guildId = channel.guild_id ?: selectedGuild?.id
            if (guildId != null) {
                val subscription = GuildSubscription(
                    guild_id = guildId,
                    channels = mapOf(channel.id to listOf(listOf(0, 99)))
                )
                gatewayManager.sendPayload(GatewayPayload(op = 14, d = json.encodeToJsonElement(subscription)))
            }
        }
    }

    fun selectThread(thread: Channel) {
        selectedThread = thread
        messages.clear()
        
        scope.launch {
            val channelMessages = discordClient.getChannelMessages(thread.id)
            messages.addAll(channelMessages)
        }
    }

    fun sendMessage(content: String) {
        val channelId = selectedChannel?.id ?: return
        scope.launch {
            discordClient.sendMessage(channelId, content)
        }
    }

    fun showProfile(userId: String) {
        selectedProfile = null
        scope.launch {
            val profile = discordClient.getUserProfile(userId, selectedGuild?.id)
            selectedProfile = profile
        }
    }
}
