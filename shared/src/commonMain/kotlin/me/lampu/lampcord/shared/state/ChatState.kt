package me.lampu.lampcord.shared.state

import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.collect
import kotlinx.serialization.json.*
import kotlin.random.Random

class ChatState(
    private val gatewayManager: GatewayManager,
    private val voiceGatewayManager: VoiceGatewayManager,
    private val discordClient: DiscordClient,
    private val json: Json,
    val readStateStore: ReadStateStore,
    val presenceStore: PresenceStore,
    val userGuildSettingsStore: UserGuildSettingsStore,
    val userStore: UserStore,
    val relationshipStore: RelationshipStore,
    val guildStore: GuildStore,
    val memberListStore: MemberListStore,
    val messageStore: MessageStore,
    val typingStore: TypingStore,
    val commandStore: CommandStore,
    val experimentStore: ExperimentStore,
    val tokenStore: TokenStore,
    val settingsStore: SettingsStore
) {
    val draftMessages = mutableStateMapOf<String, String>()

    var isConnected by mutableStateOf(false)
    var isConnecting by mutableStateOf(false)

    val typingUsers get() = typingStore.typingUsers

    val availableCommands get() = commandStore.availableCommands
    val availableApplications get() = commandStore.availableApplications

    // UI exposed state (delegating to stores where appropriate)
    val messages get() = messageStore.messages
    val guilds get() = guildStore.guilds
    val channels get() = guildStore.channels
    val privateChannels get() = guildStore.privateChannels
    val forumThreads get() = guildStore.forumThreads
    
    val readStates get() = readStateStore.readStates
    
    val memberListItems get() = memberListStore.memberListItems
    val memberListGroups get() = memberListStore.memberListGroups
    val memberListRowCount get() = memberListStore.memberListRowCount
    
    val relationships get() = relationshipStore.relationships
    val presences get() = presenceStore.presences
    
    var currentUser by userStore::currentUser
    var userSettings by settingsStore::userSettings
    
    var selectedGuild by guildStore::selectedGuild
    var selectedChannel by guildStore::selectedChannel
    var selectedThread by guildStore::selectedThread
    
    var selectedGuildOnboarding by mutableStateOf<Onboarding?>(null)
    
    var isFriendsSelected by mutableStateOf(false)
    
    val currentMember by derivedStateOf {
        val user = currentUser ?: return@derivedStateOf null
        selectedGuild?.let { userStore.getMember(it.id, user.id) } ?: selectedGuild?.members?.find { it.user?.id == user.id || (it.user == null && it.joined_at.isNotEmpty()) }
    }

    var selectedUser by mutableStateOf<User?>(null)
    var selectedProfile by mutableStateOf<UserProfile?>(null)
    var sidebarProfile by mutableStateOf<UserProfile?>(null)
    var isSidebarProfileLoading by mutableStateOf(false)
    var isProfileExpanded by mutableStateOf(false)
    var isProfileLoading by mutableStateOf(false)
    var profilePosition by mutableStateOf<Offset?>(null)
    var replyingTo by mutableStateOf<Message?>(null)
    var editingMessage by mutableStateOf<Message?>(null)
    var forwardingMessage by mutableStateOf<Message?>(null)
    var isSettingsVisible by mutableStateOf(false)
    var isQuickSwitcherVisible by mutableStateOf(false)
    var scrollToMessageId by mutableStateOf<String?>(null)
    var highlightedMessageId by mutableStateOf<String?>(null)
    val pendingFiles = mutableStateListOf<Pair<String, ByteArray>>()

    var currentVoiceState by mutableStateOf<VoiceState?>(null)
    var isVoiceConnected by mutableStateOf(false)
    var voiceConnectionDuration by mutableStateOf(0L)
    private var voiceTimerJob: Job? = null
    val voiceStates = mutableStateMapOf<String, SnapshotStateMap<String, VoiceState>>() // guildId -> userId -> VoiceState

    var isVoiceChatTextVisible by mutableStateOf(false)
    var isChannelsAndRolesVisible by mutableStateOf(false)
    var isServerSettingsVisible by mutableStateOf(false)
    var isMediaPickerVisible by mutableStateOf(false)
    var isPinsVisible by mutableStateOf(false)
    val pinnedMessages = mutableStateListOf<Message>()

    var activeCommand by mutableStateOf<ApplicationCommand?>(null)
    val commandOptions = mutableStateMapOf<String, JsonElement>()

    // Autocomplete state
    var autocompleteType by mutableStateOf<AutocompleteType?>(null)
    var autocompleteQuery by mutableStateOf("")
    var autocompleteSelectedIndex by mutableStateOf(0)
    val autocompleteItems = mutableStateListOf<AutocompleteItem>()

    fun updateAutocomplete(type: AutocompleteType?, query: String) {
        if (type == null) {
            autocompleteType = null
            autocompleteQuery = ""
            autocompleteItems.clear()
            return
        }

        autocompleteType = type
        autocompleteQuery = query
        
        val results = mutableListOf<AutocompleteItem>()
        when (type) {
            AutocompleteType.MENTION -> {
                val guildId = selectedGuild?.id
                if (query.isEmpty() || "everyone".contains(query, ignoreCase = true)) {
                    results.add(AutocompleteItem(id = "everyone", title = "everyone", replacement = "@everyone", iconType = Icons.Filled.Group))
                }
                if (query.isEmpty() || "here".contains(query, ignoreCase = true)) {
                    results.add(AutocompleteItem(id = "here", title = "here", replacement = "@here", iconType = Icons.Filled.Group))
                }

                val members = if (guildId != null) {
                     memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.filter {
                         val name = it.nick ?: it.user?.global_name ?: it.user?.username ?: ""
                         name.contains(query, ignoreCase = true) || it.user?.username?.contains(query, ignoreCase = true) == true
                     }.take(10)
                } else {
                    relationshipStore.relationships.filter { 
                        it.user?.global_name?.contains(query, ignoreCase = true) == true || 
                        it.user?.username?.contains(query, ignoreCase = true) == true
                    }.mapNotNull { it.user?.let { u -> Member(user = u) } }.take(10)
                }
                
                results.addAll(members.map { member ->
                    val user = member.user!!
                    val name = member.nick ?: user.global_name ?: user.username ?: "Unknown User"
                    AutocompleteItem(
                        id = user.id,
                        title = name,
                        subtitle = user.username,
                        icon = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" },
                        replacement = "<@${user.id}>"
                    )
                })

                val roles = selectedGuild?.roles?.filter { 
                    it.name.contains(query, ignoreCase = true) 
                }?.take(5) ?: emptyList()
                results.addAll(roles.map { role ->
                    AutocompleteItem(
                        id = role.id,
                        title = role.name,
                        iconType = Icons.Filled.Group,
                        replacement = "<@&${role.id}>",
                        color = if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else null
                    )
                })
            }
            AutocompleteType.CHANNEL -> {
                val channels = guildStore.channels.filter { 
                    it.type in listOf(0, 2, 4, 5, 13, 15, 16) && it.name?.contains(query, ignoreCase = true) == true 
                }.take(10)
                results.addAll(channels.map { channel ->
                    AutocompleteItem(
                        id = channel.id,
                        title = channel.name ?: "unnamed",
                        iconType = when (channel.type) {
                            4 -> Icons.Filled.Folder
                            15 -> Icons.Outlined.Forum
                            2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                            5 -> Icons.Filled.Campaign
                            else -> Icons.Filled.Tag
                        },
                        replacement = if (channel.type == 4) channel.name ?: "" else "<#${channel.id}>"
                    )
                })
            }
            AutocompleteType.COMMAND -> {
                val commands = commandStore.availableCommands.filter { 
                    it.name.contains(query, ignoreCase = true) 
                }.take(10)
                results.addAll(commands.map { command ->
                    val app = commandStore.availableApplications.find { it.id == command.application_id }
                    AutocompleteItem(
                        id = command.id,
                        title = "/${command.name}",
                        subtitle = command.description,
                        icon = app?.icon?.let { "https://cdn.discordapp.com/app-icons/${app.id}/$it.png?size=64" },
                        replacement = command.name,
                        isCommand = true,
                        commandObj = command
                    )
                })
            }
            AutocompleteType.EMOJI -> {
                val emojis = selectedGuild?.emojis?.filter { 
                    it.name?.contains(query, ignoreCase = true) == true 
                }?.take(15) ?: emptyList()
                results.addAll(emojis.map { emoji ->
                    AutocompleteItem(
                        id = emoji.id ?: emoji.name ?: "",
                        title = ":${emoji.name}:",
                        icon = if (emoji.id != null) "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=64" else null,
                        replacement = if (emoji.id != null) "<:${emoji.name}:${emoji.id}>" else ":${emoji.name}:"
                    )
                })

                // Add standard emojis
                if (results.size < 20) {
                    val standardEmojis = me.lampu.lampcord.shared.utils.EmojiIndex.getAllEmojis().filter {
                        it.name?.contains(query, ignoreCase = true) == true
                    }.take(20 - results.size)
                    results.addAll(standardEmojis.map { emoji ->
                        AutocompleteItem(
                            id = emoji.name ?: "",
                            title = ":${emoji.name}:",
                            icon = emoji.url,
                            replacement = me.lampu.lampcord.shared.utils.EmojiIndex.getCharForName(emoji.name ?: "") ?: emoji.name ?: ""
                        )
                    })
                }
            }
            AutocompleteType.ROLE -> { }
        }

        autocompleteItems.clear()
        autocompleteItems.addAll(results)
        autocompleteSelectedIndex = 0
    }

    // Attachment viewer state
    var isAttachmentViewerVisible by mutableStateOf(false)
    var attachmentViewerItems by mutableStateOf<List<DiscordMedia>>(emptyList())
    var attachmentViewerIndex by mutableStateOf(0)

    fun openAttachmentViewer(items: List<DiscordMedia>, index: Int) {
        if (items.isEmpty()) return
        attachmentViewerItems = items
        attachmentViewerIndex = index.coerceIn(0, items.lastIndex)
        isAttachmentViewerVisible = true
    }

    fun closeAttachmentViewer() {
        isAttachmentViewerVisible = false
        attachmentViewerItems = emptyList()
        attachmentViewerIndex = 0
    }
    
    private var currentToken: String? = null
    private var currentFingerprint: String? = null

    private var guildLoadingJob: kotlinx.coroutines.Job? = null
    private var channelLoadingJob: kotlinx.coroutines.Job? = null
    private var historyLoadingJob: kotlinx.coroutines.Job? = null
    
    var isLoadingHistory by messageStore::isLoadingHistory
    var hasMoreHistory by messageStore::hasMoreHistory
    var isForumLoading by mutableStateOf(false)

    private val scope = CoroutineScope(Dispatchers.Main)

    init {
        scope.launch {
            me.lampu.lampcord.shared.utils.EmojiIndex.initialize()
        }
    }

    private val subscribedGuilds = mutableSetOf<String>()
    private var lastRequestedKey: String? = null

    init {
        val savedToken = Settings.shared.discordToken
        if (savedToken.isNotBlank()) {
            connect(savedToken)
        }

        scope.launch {
            gatewayManager.events.collect { payload ->
                handleGatewayEvent(payload)
            }
        }
    }

    private fun handleGatewayEvent(payload: GatewayPayload) {
        when (payload.t) {
            "READY" -> handleReady(payload)
            "RESUMED" -> {
                isConnected = true
                isConnecting = false
                println("Session resumed successfully")
            }
            "GUILD_CREATE" -> handleGuildCreate(payload)
            "GUILD_UPDATE" -> handleGuildUpdate(payload)
            "GUILD_DELETE" -> handleGuildDelete(payload)
            "CHANNEL_CREATE", "CHANNEL_UPDATE" -> handleChannelUpdate(payload)
            "CHANNEL_DELETE" -> handleChannelDelete(payload)
            "GUILD_ROLE_CREATE", "GUILD_ROLE_UPDATE" -> handleRoleUpdate(payload)
            "GUILD_ROLE_DELETE" -> handleRoleDelete(payload)
            "MESSAGE_CREATE" -> handleMessageCreate(payload)
            "GUILD_MEMBER_LIST_UPDATE" -> handleMemberListUpdate(payload)
            "THREAD_LIST_SYNC" -> handleThreadListSync(payload)
            "THREAD_CREATE", "THREAD_UPDATE" -> handleThreadUpdate(payload)
            "THREAD_DELETE" -> handleThreadDelete(payload)
            "MESSAGE_UPDATE" -> handleMessageUpdate(payload)
            "MESSAGE_DELETE" -> handleMessageDelete(payload)
            "RELATIONSHIP_ADD" -> handleRelationshipAdd(payload)
            "RELATIONSHIP_REMOVE" -> handleRelationshipRemove(payload)
            "USER_SETTINGS_UPDATE" -> handleUserSettingsUpdate(payload)
            "PRESENCE_UPDATE" -> handlePresenceUpdate(payload)
            "USER_UPDATE" -> handleUserUpdate(payload)
            "USER_NOTE_UPDATE" -> handleUserNoteUpdate(payload)
            "TYPING_START" -> handleTypingStart(payload)
            "MESSAGE_ACK" -> handleMessageAck(payload)
            "MESSAGE_REACTION_ADD" -> handleReactionAddEvent(payload)
            "MESSAGE_REACTION_REMOVE" -> handleReactionRemoveEvent(payload)
            "USER_GUILD_SETTINGS_UPDATE" -> handleUserGuildSettingsUpdate(payload)
            "VOICE_STATE_UPDATE" -> handleVoiceStateUpdate(payload)
            "VOICE_SERVER_UPDATE" -> handleVoiceServerUpdate(payload)
        }
    }

    private fun handleVoiceStateUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val state = json.decodeFromJsonElement<VoiceState>(data)
                println("Voice State Update: User=${state.user_id}, Channel=${state.channel_id}, Guild=${state.guild_id}")
                val guildId = state.guild_id ?: "@me"
                val guildMap = voiceStates.getOrPut(guildId) { mutableStateMapOf() }
                
                if (state.channel_id == null) {
                    guildMap.remove(state.user_id)
                } else {
                    guildMap[state.user_id] = state
                }

                if (state.user_id == currentUser?.id) {
                    if (state.channel_id == null) {
                        currentVoiceState = null
                        isVoiceConnected = false
                        stopVoiceTimer()
                    } else {
                        val isNewConnection = currentVoiceState == null
                        currentVoiceState = state
                        isVoiceConnected = true
                        if (isNewConnection) startVoiceTimer()
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun startVoiceTimer() {
        voiceTimerJob?.cancel()
        voiceConnectionDuration = 0L
        voiceTimerJob = scope.launch {
            while (isActive) {
                delay(1000)
                voiceConnectionDuration++
            }
        }
    }

    private fun stopVoiceTimer() {
        voiceTimerJob?.cancel()
        voiceTimerJob = null
        voiceConnectionDuration = 0L
    }

    private fun handleVoiceServerUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val update = json.decodeFromJsonElement<VoiceServerUpdate>(data)
                val userId = currentUser?.id ?: return
                val sessionId = currentVoiceState?.session_id ?: return
                
                if (update.endpoint != null) {
                    // voiceGatewayManager.connect(
                    //     endpoint = update.endpoint,
                    //     guildId = update.guild_id,
                    //     userId = userId,
                    //     sessionId = sessionId,
                    //     token = update.token
                    // )
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleUserGuildSettingsUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val settings = json.decodeFromJsonElement<UserGuildSettings>(data)
                userGuildSettingsStore.handleUpdate(settings)
            } catch (e: Exception) { }
        }
    }

    private fun handleReady(payload: GatewayPayload) {
        payload.d?.let { data ->
            println("Received READY payload, decoding...")

            // 1. Cache users first so they are available for hydration
            try {
                data.jsonObject["users"]?.jsonArray?.forEach {
                    val u = json.decodeFromJsonElement<User>(it)
                    userStore.handleUserUpdate(u)
                }
            } catch (e: Exception) {
                println("Failed to pre-cache users: ${e.message}")
            }

            val user = try {
                data.jsonObject["user"]?.let { json.decodeFromJsonElement<User>(it) }
            } catch (e: Exception) {
                println("Failed to decode user from READY: ${e.message}")
                null
            }

            user?.let { u ->
                currentUser = u
                userStore.handleUserUpdate(u)
                currentToken?.let { t ->
                    tokenStore.addAccount(t, u)
                    Settings.shared.discordToken = t
                }
            }

            try {
                val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                println("Successfully decoded ReadyPayload with ${ready.guilds.size} guilds and ${ready.private_channels.size} DMs")

                if (ready.user_settings != null) {
                    ready.user_settings.let { el ->
                        try {
                            if (el is JsonObject) {
                                val settings = json.decodeFromJsonElement<UserSettings>(el)
                                settingsStore.userSettings = settings
                                println("Decoded UserSettings successfully with ${settings.guild_folders.size} folders")
                            } else if (el is JsonPrimitive && el.isString) {
                                println("UserSettings is an encoded string, skipping for now.")
                            }
                        } catch (e: Exception) {
                            println("Failed to decode UserSettings: ${e.message}")
                        }
                    }
                }
                
                userGuildSettingsStore.handleReady(ready)
                
                // Hydrate private channels recipients using the userStore we just populated
                val hydratedPrivateChannels = ready.private_channels.map { channel ->
                    val recipients = (channel.recipients ?: channel.recipient_ids?.map { User(id = it) })?.map { r ->
                        userStore.getUser(r.id) ?: r
                    }
                    channel.copy(recipients = recipients)
                }
                
                val guildOrder = settingsStore.userSettings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() } ?: emptyList()
                guildStore.setGuilds(ready.guilds, guildOrder)
                guildStore.setPrivateChannels(hydratedPrivateChannels.sortedByDescending { it.lastMessageId() ?: "0" })
                
                readStateStore.handleReady(ready)
                experimentStore.handleReady(ready.experiments)

                // Handle merged members
                ready.merged_members?.forEachIndexed { index, members ->
                    val guildId = ready.guilds.getOrNull(index)?.id ?: return@forEachIndexed
                    members.forEach { member ->
                        member.user?.let { user ->
                            userStore.cacheMember(guildId, user.id, member)
                        }
                    }
                }

                // Handle relationships
                ready.relationships?.let { rels ->
                    relationshipStore.handleReady(rels)
                }

                // Handle merged presences
                ready.merged_presences?.guilds?.forEachIndexed { index, presences ->
                    presences.forEach { presence ->
                        presenceStore.handlePresenceUpdate(presence)
                    }
                }
                ready.merged_presences?.friends?.forEach { presence ->
                    presenceStore.handlePresenceUpdate(presence)
                }

                // Parity with 126.21: Clear voice state on READY
                gatewayManager.sendVoiceStateUpdate(
                    guildId = null,
                    channelId = null,
                    selfMute = true,
                    selfDeaf = true
                )
                
                // Parity with 126.21: Don't fetch relationships via REST if they are in READY
                // relationshipStore.fetchRelationships()
                
                // Background fetch all channels for forwarding/switcher cache
                scope.launch {
                    ready.guilds.forEach { guild ->
                        if (guildStore.allGuildChannels[guild.id] == null) {
                            try {
                                val gChannels = discordClient.getGuildChannels(guild.id)
                                if (gChannels.isNotEmpty()) {
                                    guildStore.allGuildChannels[guild.id] = gChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
                                }
                                delay(2000)
                            } catch (e: Exception) { }
                        }
                    }
                }

                isConnected = true
                isConnecting = false

                if (selectedGuild == null && selectedChannel == null) {
                    selectHome()
                }
            } catch (e: Exception) {
                println("Error decoding READY: ${e.message}")
            }
        }
    }

    private fun handleGuildCreate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val guild = json.decodeFromJsonElement<Guild>(data)
                println("Received GUILD_CREATE for ${guild.id}, name: ${guild.name}, icon: ${guild.icon}")
                val guildOrder = settingsStore.userSettings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() } ?: emptyList()
                guildStore.handleGuildCreate(guild, guildOrder)
                
                guild.members?.forEach { member ->
                    member.user?.let { user ->
                        userStore.cacheMember(guild.id, user.id, member)
                    }
                }
            } catch (e: Exception) { 
                println("Error decoding GUILD_CREATE: ${e.message}")
            }
        }
    }

    private fun handleGuildUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val guild = json.decodeFromJsonElement<Guild>(data)
                println("Received GUILD_UPDATE for ${guild.id}, name: ${guild.name}")
                val guildOrder = settingsStore.userSettings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() } ?: emptyList()
                guildStore.handleGuildCreate(guild, guildOrder)
            } catch (e: Exception) { }
        }
    }

    private fun handleGuildDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return
                guildStore.handleGuildDelete(id)
            } catch (e: Exception) { }
        }
    }

    private fun handleChannelUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val channel = json.decodeFromJsonElement<Channel>(data)
                guildStore.handleChannelCreateOrUpdate(channel)
            } catch (e: Exception) { }
        }
    }

    private fun handleChannelDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val channel = json.decodeFromJsonElement<Channel>(data)
                guildStore.handleChannelDelete(channel)
            } catch (e: Exception) { }
        }
    }

    private fun handleRoleUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val update = json.decodeFromJsonElement<GuildRoleUpdate>(data)
                guildStore.handleRoleCreateOrUpdate(update.guild_id, update.role)
            } catch (e: Exception) { }
        }
    }

    private fun handleRoleDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val delete = json.decodeFromJsonElement<GuildRoleDelete>(data)
                guildStore.handleRoleDelete(delete.guild_id, delete.role_id)
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageCreate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val message = json.decodeFromJsonElement<Message>(data)

                message.guild_id?.let { guildId ->
                    message.member?.let { member ->
                        message.author?.let { author ->
                            userStore.cacheMember(guildId, author.id, member)
                        }
                    }
                }
                message.author?.let { userStore.handleUserUpdate(it) }
                
                val privateChannels = guildStore.privateChannels
                val dmIndex = privateChannels.indexOfFirst { it.id == message.channel_id }
                if (dmIndex != -1) {
                    val dm = privateChannels.removeAt(dmIndex)
                    privateChannels.add(0, dm.copy(last_message_id = JsonPrimitive(message.id)))
                }

                if (selectedChannel?.id == message.channel_id || selectedThread?.id == message.channel_id) {
                    messageStore.handleMessageCreate(message)
                    scope.launch {
                        readStateStore.ackMessage(message.channel_id, message.id)
                    }
                } else {
                    if (isMessageMentioningMe(message)) {
                        val state = readStates[message.channel_id]
                        if (state != null) {
                            readStateStore.readStates[message.channel_id] = state.copy(mention_count = state.mention_count + 1)
                        } else {
                            readStateStore.readStates[message.channel_id] = ReadState(id = message.channel_id, mention_count = 1)
                        }
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleMemberListUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val update = json.decodeFromJsonElement<MemberListUpdate>(data)
                if (update.guild_id == selectedGuild?.id) {
                    memberListStore.handleMemberListUpdate(update)
                    update.ops.forEach { op ->
                        op.items?.forEach { it.member?.let { m -> 
                            val userId = m.user?.id ?: m.presence?.user?.id ?: return@let
                            userStore.cacheMember(update.guild_id, userId, m)
                            m.presence?.let { p -> presenceStore.handlePresenceUpdate(p) }
                        } }
                        op.item?.member?.let { m -> 
                            val userId = m.user?.id ?: m.presence?.user?.id ?: return@let
                            userStore.cacheMember(update.guild_id, userId, m)
                            m.presence?.let { p -> presenceStore.handlePresenceUpdate(p) }
                        }
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleThreadListSync(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val sync = json.decodeFromJsonElement<ThreadListSync>(data)
                val forum = selectedChannel ?: return
                if (forum.type == 15 && sync.guild_id == selectedGuild?.id) {
                    if (sync.channel_ids == null || forum.id in sync.channel_ids) {
                        sync.threads.filter { it.parent_id == forum.id }.forEach { upsertForumThread(it) }
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleThreadUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val thread = json.decodeFromJsonElement<Channel>(data)
                upsertForumThread(thread)
            } catch (e: Exception) { }
        }
    }

    private fun handleThreadDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val delete = json.decodeFromJsonElement<ThreadDeleteEvent>(data)
                val forum = selectedChannel ?: return
                if (forum.type == 15 && delete.parent_id == forum.id) {
                    guildStore.forumThreads.removeAll { it.id == delete.id }
                    if (selectedThread?.id == delete.id) selectedThread = null
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val partialMessage = json.decodeFromJsonElement<Message>(data)
                messageStore.handleMessageUpdate(partialMessage, data.jsonObject)
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return@let
                messageStore.handleMessageDelete(id)
            } catch (e: Exception) { }
        }
    }

    private fun handleRelationshipAdd(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                relationshipStore.handleRelationshipAdd(json.decodeFromJsonElement<Relationship>(data))
            } catch (e: Exception) { }
        }
    }

    private fun handleRelationshipRemove(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return@let
                relationshipStore.handleRelationshipRemove(id)
            } catch (e: Exception) { }
        }
    }

    private fun handleUserSettingsUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val newSettings = json.decodeFromJsonElement<UserSettings>(data)
                settingsStore.handleUserSettingsUpdate(newSettings)
                val guildOrder = newSettings.guild_positions.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() }
                guildStore.setGuilds(guilds, guildOrder)
            } catch (e: Exception) { }
        }
    }

    private fun handlePresenceUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                presenceStore.handlePresenceUpdate(json.decodeFromJsonElement<PresenceUpdate>(data))
            } catch (e: Exception) { }
        }
    }

    private fun handleUserUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val user = json.decodeFromJsonElement<User>(data)
                userStore.handleUserUpdate(user)
            } catch (e: Exception) { }
        }
    }

    private fun handleUserNoteUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val update = json.decodeFromJsonElement<UserNoteUpdate>(data)
                // We don't have a note store yet, but we can log it or add it to UserProfile
                println("User note updated for ${update.id}: ${update.note}")
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageAck(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                readStateStore.handleMessageAck(json.decodeFromJsonElement<MessageAcknowledge>(data))
            } catch (e: Exception) { }
        }
    }

    private fun handleTypingStart(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val typing = json.decodeFromJsonElement<TypingStart>(data)
                typingStore.handleTypingStart(typing.channel_id, typing.user_id, currentUser?.id)
            } catch (e: Exception) { }
        }
    }

    private fun handleReactionAddEvent(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val add = json.decodeFromJsonElement<MessageReactionAdd>(data)
                val index = messages.indexOfFirst { it.id == add.message_id }
                if (index != -1) {
                    val msg = messages[index]
                    val reactions = msg.reactions?.toMutableList() ?: mutableListOf()
                    val emojiIndex = reactions.indexOfFirst { it.emoji.id == add.emoji.id && it.emoji.name == add.emoji.name }
                    val isMe = add.user_id == currentUser?.id
                    if (emojiIndex != -1) {
                        val r = reactions[emojiIndex]
                        reactions[emojiIndex] = r.copy(count = r.count + 1, me = if (isMe) true else r.me)
                    } else {
                        reactions.add(MessageReaction(emoji = add.emoji, count = 1, me = isMe, me_burst = false, count_details = ReactionCountDetails(0, 1)))
                    }
                    messages[index] = msg.copy(reactions = reactions)
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleReactionRemoveEvent(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val remove = json.decodeFromJsonElement<MessageReactionRemove>(data)
                val index = messages.indexOfFirst { it.id == remove.message_id }
                if (index != -1) {
                    val msg = messages[index]
                    val reactions = msg.reactions?.toMutableList() ?: return
                    val emojiIndex = reactions.indexOfFirst { it.emoji.id == remove.emoji.id && it.emoji.name == remove.emoji.name }
                    if (emojiIndex != -1) {
                        val r = reactions[emojiIndex]
                        val isMe = remove.user_id == currentUser?.id
                        val newCount = (r.count - 1).coerceAtLeast(0)
                        if (newCount == 0) {
                            reactions.removeAt(emojiIndex)
                        } else {
                            reactions[emojiIndex] = r.copy(count = newCount, me = if (isMe) false else r.me)
                        }
                        messages[index] = msg.copy(reactions = reactions)
                    }
                }
            } catch (e: Exception) { }
        }
    }

    suspend fun loadForumThreads(channelId: String): List<Channel> {
        val all = mutableListOf<Channel>()
        discordClient.getActiveThreads(channelId)?.threads?.let { all.addAll(it) }
        
        var before: String? = null
        while (true) {
            val archived = discordClient.getArchivedPublicThreads(channelId, before = before) ?: break
            if (archived.threads.isEmpty()) break
            all.addAll(archived.threads)
            if (archived.has_more != true) break
            before = archived.threads.last().thread_metadata?.archive_timestamp ?: break
        }
        return all.sortedByDescending { it.forumSortKey() }
    }

    fun upsertForumThread(thread: Channel) {
        val index = forumThreads.indexOfFirst { it.id == thread.id }
        if (index != -1) {
            guildStore.forumThreads[index] = thread
        } else {
            guildStore.forumThreads.add(thread)
        }
        val sorted = forumThreads.sortedByDescending { it.forumSortKey() }
        guildStore.forumThreads.clear()
        guildStore.forumThreads.addAll(sorted)
    }

    private fun Channel.forumSortKey(): Long = last_message_id?.jsonPrimitive?.contentOrNull?.toLongOrNull() ?: id.toLong()

    fun connect(token: String) {
        currentToken = token
        discordClient.setToken(token)
        isConnecting = true
        gatewayManager.connect(token)
    }

    fun disconnect() {
        gatewayManager.disconnect()
        voiceGatewayManager.disconnect()
        isConnected = false
        isConnecting = false
        currentToken = null
        discordClient.setToken(null)
        clearAllStores()
    }

    fun clearAllStores() {
        readStateStore.clear()
        userGuildSettingsStore.clear()
        presenceStore.clear()
        userStore.clear()
        relationshipStore.clear()
        guildStore.clear()
        memberListStore.clear()
        messageStore.clear()
        typingStore.clear()
        commandStore.clear()
    }

    suspend fun login(email: String, pass: String): LoginResponse? {
        val fingerprint = discordClient.getFingerprint() ?: ""
        currentFingerprint = fingerprint
        return discordClient.login(LoginRequest(email, pass), fingerprint)
    }

    suspend fun verifyMFA(ticket: String, code: String, type: String): Boolean {
        val fingerprint = currentFingerprint ?: discordClient.getFingerprint() ?: ""
        val res = discordClient.loginMFA(MFALoginRequest(ticket, code), fingerprint, type)
        return if (res?.token != null) {
            connect(res.token)
            true
        } else false
    }

    fun selectGuild(guild: Guild) {
        if (selectedGuild?.id == guild.id && !isChannelsAndRolesVisible) return
        guildLoadingJob?.cancel()
        selectedGuild = guild
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        guildStore.channels.clear()
        memberListStore.clear()
        lastRequestedKey = null
        guildLoadingJob = scope.launch {
            subscribeToGuild(guild.id)
            val guildChannels = discordClient.getGuildChannels(guild.id)
            if (guildChannels.isNotEmpty()) {
                val filtered = guildChannels.filter { it.type in listOf(0, 2, 5, 4, 13, 15, 16) }.sortedBy { it.position }
                guildStore.allGuildChannels[guild.id] = filtered
                guildStore.channels.clear()
                guildStore.channels.addAll(filtered)
            }
            val lastChannelId = Settings.shared.getLastChannel(guild.id)
            val channelToSelect = if (lastChannelId != null) channels.find { it.id == lastChannelId } else channels.firstOrNull { it.type in listOf(0, 2, 5, 13, 15) }
            channelToSelect?.let { selectChannel(it) }
            
            // Onboarding
            selectedGuildOnboarding = null
            if (guild.features?.contains("ONBOARDING") == true) {
                selectedGuildOnboarding = discordClient.getGuildOnboarding(guild.id)
            }
            
            // Fetch commands
            try {
                val index = discordClient.getCommandIndex(guild.id)
                commandStore.clear()
                index?.let {
                    commandStore.setCommands(it.application_commands, it.applications)
                }
            } catch (e: Exception) { }
        }
    }

    fun isUnread(channel: Channel): Boolean {
        val isMuted = userGuildSettingsStore.isChannelMuted(channel.guild_id, channel.id)
        if (isMuted) return false
        return readStateStore.isUnread(channel)
    }

    fun isGuildUnread(guildId: String): Boolean {
        if (userGuildSettingsStore.isGuildMuted(guildId)) return false
        return guildStore.allGuildChannels[guildId]?.any { isUnread(it) } ?: false
    }

    fun isFolderUnread(folder: GuildFolder): Boolean {
        return folder.guild_ids.any { el -> 
            val id = el.jsonPrimitive.contentOrNull ?: el.toString()
            isGuildUnread(id) 
        }
    }
    
    fun getGuildMentionCount(guildId: String): Int {
        return guildStore.allGuildChannels[guildId]?.sumOf { getMentionCount(it.id) } ?: 0
    }
    
    fun getFolderMentionCount(folder: GuildFolder): Int {
        return folder.guild_ids.sumOf { el ->
            val id = el.jsonPrimitive.contentOrNull ?: el.toString()
            getGuildMentionCount(id)
        }
    }

    fun getMentionCount(channelId: String) = readStateStore.getMentionCount(channelId)

    fun selectHome() {
        selectedGuild = null
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        memberListStore.clear()
        lastRequestedKey = null
        val lastDmId = Settings.shared.getLastChannel("home")
        if (lastDmId == "friends") {
            isFriendsSelected = true
            selectedChannel = null
        } else {
            val dmToSelect = if (lastDmId != null) privateChannels.find { it.id == lastDmId } else privateChannels.firstOrNull()
            if (dmToSelect != null) {
                selectChannel(dmToSelect)
            } else {
                isFriendsSelected = false
                selectedChannel = null
            }
        }
    }

    fun selectFriends() {
        selectedGuild = null
        selectedChannel = null
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        isFriendsSelected = true
        Settings.shared.setLastChannel("home", "friends")
    }

    fun selectChannel(channel: Channel) {
        if (selectedChannel?.id == channel.id) return
        isFriendsSelected = false
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        isVoiceChatTextVisible = false
        channelLoadingJob?.cancel()
        selectedChannel = channel
        selectedThread = null

        // 126.21 Parity: Pre-size the member list store based on expected list ID
        val guild = selectedGuild
        if (guild != null) {
            val expectedId = channel.member_list_id ?: channel.memberListId(guild)
            memberListStore.setExpectedId(expectedId, guild.member_count ?: 0)
        } else {
            memberListStore.clear()
        }

        messageStore.clear()
        lastRequestedKey = null
        Settings.shared.setLastChannel(selectedGuild?.id ?: "home", channel.id)

        // 126.21 Parity: Request member list range IMMEDIATELY, don't wait for messages/profile
        if (channel.guild_id != null || selectedGuild != null) requestMemberListRange(listOf(listOf(0, 99)))
        
        channelLoadingJob = scope.launch {
            if (channel.type == 1) {
                val userId = channel.recipients?.firstOrNull()?.id
                if (userId != null) {
                    sidebarProfile = null
                    isSidebarProfileLoading = true
                    sidebarProfile = discordClient.getUserProfile(userId)
                    isSidebarProfileLoading = false
                }
            } else {
                sidebarProfile = null
                isSidebarProfileLoading = false
            }

            if (channel.type == 15) {
                isForumLoading = true
                try { forumThreads.clear(); guildStore.forumThreads.addAll(loadForumThreads(channel.id)) } finally { isForumLoading = false }
            } else {
                val channelMessages = discordClient.getChannelMessages(channel.id)
                channelMessages.forEach { msg -> 
                    msg.author?.let { author ->
                        msg.member?.let { m -> userStore.cacheMember(channel.guild_id ?: selectedGuild?.id ?: "", author.id, m) }
                    }
                }
                messageStore.addMessages(channelMessages)
                channelMessages.firstOrNull()?.let { readStateStore.ackMessage(channel.id, it.id) }
            }
        }
    }

    fun selectThread(channel: Channel) {
        if (selectedThread?.id == channel.id) return
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        isVoiceChatTextVisible = false
        selectedThread = channel
        messageStore.clear()
        scope.launch {
            val channelMessages = discordClient.getChannelMessages(channel.id)
            channelMessages.forEach { msg -> 
                msg.author?.let { author ->
                    msg.member?.let { m -> userStore.cacheMember(channel.guild_id ?: selectedGuild?.id ?: "", author.id, m) }
                }
            }
            messageStore.addMessages(channelMessages)
            channelMessages.firstOrNull()?.let { readStateStore.ackMessage(channel.id, it.id) }
        }
    }

    fun connectToVoice(channel: Channel) {
        gatewayManager.sendVoiceStateUpdate(
            guildId = channel.guild_id,
            channelId = channel.id,
            selfMute = currentVoiceState?.self_mute ?: false,
            selfDeaf = currentVoiceState?.self_deaf ?: false,
            selfVideo = false
        )
    }

    fun disconnectFromVoice() {
        val guildId = currentVoiceState?.guild_id ?: selectedGuild?.id
        println("Disconnecting from voice: Guild=$guildId")
        gatewayManager.sendVoiceStateUpdate(
            guildId = guildId,
            channelId = null,
            selfMute = true,
            selfDeaf = true
        )
        voiceGatewayManager.disconnect()
    }

    fun toggleVoiceMute() {
        val state = currentVoiceState ?: return
        gatewayManager.sendVoiceStateUpdate(
            guildId = state.guild_id,
            channelId = state.channel_id,
            selfMute = !state.self_mute,
            selfDeaf = state.self_deaf,
            selfVideo = state.self_video
        )
    }

    fun toggleVoiceDeaf() {
        val state = currentVoiceState ?: return
        gatewayManager.sendVoiceStateUpdate(
            guildId = state.guild_id,
            channelId = state.channel_id,
            selfMute = state.self_mute,
            selfDeaf = !state.self_deaf,
            selfVideo = state.self_video
        )
    }

    fun toggleVoiceVideo() {
        val state = currentVoiceState ?: return
        gatewayManager.sendVoiceStateUpdate(
            guildId = state.guild_id,
            channelId = state.channel_id,
            selfMute = state.self_mute,
            selfDeaf = state.self_deaf,
            selfVideo = !state.self_video
        )
    }

    fun toggleVoiceStream() {
        val state = currentVoiceState ?: return
        // selfStream is not in the VoiceStateUpdate payload
    }

    fun loadMoreMessages() {
        if (isLoadingHistory || !hasMoreHistory || selectedChannel == null) return
        val before = messages.lastOrNull()?.id ?: return
        isLoadingHistory = true
        historyLoadingJob = scope.launch {
            try {
                val more = discordClient.getChannelMessages(selectedThread?.id ?: selectedChannel!!.id, before = before)
                if (more.isEmpty()) {
                    hasMoreHistory = false
                } else {
                    more.forEach { msg -> 
                        msg.author?.let { author ->
                            msg.member?.let { m -> userStore.cacheMember(selectedGuild?.id ?: "", author.id, m) }
                        }
                    }
                    messageStore.addMessages(more)
                }
            } finally {
                isLoadingHistory = false
            }
        }
    }

    fun sendMessage(content: String) {
        val channel = selectedThread ?: selectedChannel ?: return
        messageStore.sendMessage(
            channelId = channel.id,
            content = content,
            currentUser = currentUser!!,
            replyTo = replyingTo?.id,
            files = pendingFiles.toList(),
            guildId = selectedGuild?.id,
            forwardFrom = null
        )
        replyingTo = null
        pendingFiles.clear()
        
        // Clear draft
        draftMessages.remove(channel.id)
    }

    fun forwardMessage(targetChannel: Channel, message: Message) {
        messageStore.sendMessage(
            channelId = targetChannel.id,
            content = "",
            currentUser = currentUser!!,
            replyTo = null,
            files = emptyList(),
            guildId = targetChannel.guild_id,
            forwardFrom = message
        )
    }

    fun retryMessage(message: Message) = messageStore.retryMessage(message)
    fun deletePendingMessage(message: Message) = messageStore.deletePendingMessage(message)

    private var lastTypingTime = 0L

    fun sendTyping() {
        val channel = selectedChannel ?: return
        val now = getCurrentTimeMillis()
        if (now - lastTypingTime > 8000) {
            lastTypingTime = now
            scope.launch {
                discordClient.sendTyping(channel.id)
            }
        }
    }

    fun showProfile(userId: String, position: Offset? = null) {
        selectedProfile = null
        isProfileExpanded = false
        profilePosition = position
        isProfileLoading = true
        scope.launch {
            selectedProfile = discordClient.getUserProfile(userId, selectedGuild?.id)
            isProfileLoading = false
        }
    }

    fun getUserStatus(userId: String): String = presenceStore.getUserStatus(userId, currentUser?.id, userSettings?.status)
    fun updateStatus(status: String) = scope.launch { discordClient.updateStatus(status) }
    fun updateCustomStatus(text: String?) = scope.launch { discordClient.updateCustomStatus(text) }

    fun leaveGuild(guildId: String) = scope.launch {
        if (discordClient.leaveGuild(guildId)) {
            guildStore.handleGuildDelete(guildId)
            if (selectedGuild?.id == guildId) {
                selectHome()
            }
        }
    }

    fun markGuildAsRead(guildId: String = selectedGuild?.id ?: "") = scope.launch {
        discordClient.ackBulk(listOf(guildId))
    }

    fun markCategoryAsRead(categoryId: String, guildId: String? = selectedGuild?.id) = scope.launch {
        // Implementation for marking category as read
    }

    fun hasPermission(permission: Permission, channel: Channel? = selectedChannel): Boolean {
        val member = currentMember ?: return true
        val guild = selectedGuild ?: return true
        return PermissionHelper.hasPermission(member, guild, channel, permission, currentUser?.id)
    }

    fun getMember(guildId: String, userId: String): Member? = userStore.getMember(guildId, userId)

    fun sendInteraction(command: ApplicationCommand, options: List<InteractionOption>? = null) {
        val guildId = selectedGuild?.id
        val channelId = selectedChannel?.id ?: return
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

    fun removeReaction(channelId: String, messageId: String, emoji: String) = scope.launch {
        discordClient.removeReaction(channelId, messageId, emoji)
    }
    
    fun addReaction(channelId: String, messageId: String, emoji: String) = scope.launch {
        discordClient.addReaction(channelId, messageId, emoji)
    }

    fun pinMessage(message: Message) = scope.launch {
        if (discordClient.pinMessage(message.channel_id, message.id)) {
            // Updated via gateway MESSAGE_UPDATE with pinned=true
        }
    }

    fun unpinMessage(message: Message) = scope.launch {
        if (discordClient.unpinMessage(message.channel_id, message.id)) {
            // Updated via gateway MESSAGE_UPDATE with pinned=false
            pinnedMessages.removeAll { it.id == message.id }
        }
    }

    fun showPinnedMessages() = scope.launch {
        val channel = selectedChannel ?: return@launch
        isPinsVisible = true
        pinnedMessages.clear()
        pinnedMessages.addAll(discordClient.getPinnedMessages(channel.id))
    }

    private fun subscribeToGuild(guildId: String) {
        if (guildId in subscribedGuilds) return
        subscribedGuilds.add(guildId)
        gatewayManager.sendSubscription(guildId)
    }

    fun requestMemberListRange(ranges: List<List<Int>>) {
        val guild = selectedGuild ?: return
        val channel = selectedChannel ?: return
        
        val requestKey = "${guild.id}:${channel.id}:$ranges"
        if (requestKey == lastRequestedKey) return
        lastRequestedKey = requestKey

        // 126.21 Parity: We MUST send the actual channel.id as the key in the lazy load map.
        gatewayManager.sendLazyRequest(guild.id, channel.id, ranges)
    }

    fun toggleMuteGuild(guildId: String) {
        val currentMuted = userGuildSettingsStore.isGuildMuted(guildId)
        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(muted = !currentMuted))
        }
    }

    fun toggleMuteChannel(guildId: String, channelId: String) {
        val guildSettings = userGuildSettingsStore.userGuildSettings[guildId] ?: return
        val currentOverrides = guildSettings.channel_overrides.toMutableList()
        val index = currentOverrides.indexOfFirst { it.channel_id == channelId }
        
        val newOverride = if (index != -1) {
            currentOverrides[index].copy(muted = !currentOverrides[index].muted)
        } else {
            ChannelOverride(channel_id = channelId, muted = true)
        }
        
        if (index != -1) currentOverrides[index] = newOverride else currentOverrides.add(newOverride)

        scope.launch {
            discordClient.updateUserGuildSettings(guildId, UserGuildSettings.Partial(channel_overrides = currentOverrides))
        }
    }

    fun isMessageMentioningMe(message: Message): Boolean {
        val myId = currentUser?.id ?: return false
        if (message.author?.id == myId) return false
        
        if (message.mentions.any { it.id == myId }) return true
        
        val myMember = currentMember
        if (myMember != null) {
            if (message.mention_roles.any { roleId -> roleId in myMember.roles }) return true
        }
        
        if (message.mention_everyone) return true
        
        return false
    }

    fun updateUserSettings(partial: UserSettings.Partial) {
        scope.launch {
            if (discordClient.updateUserSettings(partial)) {
                // Update local settings if success
                userSettings = userSettings?.merge(partial)
            }
        }
    }

    fun updateGuild(guildId: String, partial: Guild.Partial) {
        scope.launch {
            if (discordClient.updateGuild(guildId, partial)) {
                // Locally update guild in store
                guildStore.guilds.find { it.id == guildId }?.let { g ->
                    val updated = g.merge(partial)
                    val index = guildStore.guilds.indexOf(g)
                    if (index != -1) guildStore.guilds[index] = updated
                    if (selectedGuild?.id == guildId) selectedGuild = updated
                }
            }
        }
    }

    private fun UserSettings.merge(partial: UserSettings.Partial): UserSettings {
        return copy(
            theme = partial.theme ?: theme,
            developer_mode = partial.developer_mode ?: developer_mode,
            render_embeds = partial.render_embeds ?: render_embeds,
            inline_embed_media = partial.inline_embed_media ?: inline_embed_media,
            inline_attachment_media = partial.inline_attachment_media ?: inline_attachment_media,
            blocked_message_bar = partial.blocked_message_bar ?: blocked_message_bar,
            locale = partial.locale ?: locale,
            restricted_guilds = partial.restricted_guilds ?: restricted_guilds,
            status = partial.status ?: status,
            show_current_game = partial.show_current_game ?: show_current_game,
            guild_folders = partial.guild_folders ?: guild_folders,
            default_guilds_restricted = partial.default_guilds_restricted ?: default_guilds_restricted,
            friend_source_flags = partial.friend_source_flags ?: friend_source_flags,
            explicit_content_filter = partial.explicit_content_filter ?: explicit_content_filter,
            animate_emoji = partial.animate_emoji ?: animate_emoji,
            allow_accessibility_detection = partial.allow_accessibility_detection ?: allow_accessibility_detection,
            animate_stickers = partial.animate_stickers ?: animate_stickers,
            contact_sync_enabled = partial.contact_sync_enabled ?: contact_sync_enabled,
            friend_discovery_flags = partial.friend_discovery_flags ?: friend_discovery_flags,
            custom_status = partial.custom_status ?: custom_status
        )
    }

    private fun Guild.merge(partial: Guild.Partial): Guild {
        return copy(
            name = partial.name ?: name,
            icon = partial.icon ?: icon,
            banner = partial.banner ?: banner,
            splash = partial.splash ?: splash,
            description = partial.description ?: description,
            afk_channel_id = partial.afk_channel_id ?: afk_channel_id,
            afk_timeout = partial.afk_timeout ?: afk_timeout,
            system_channel_id = partial.system_channel_id ?: system_channel_id,
            system_channel_flags = partial.system_channel_flags ?: system_channel_flags,
            rules_channel_id = partial.rules_channel_id ?: rules_channel_id,
            public_updates_channel_id = partial.public_updates_channel_id ?: public_updates_channel_id,
            preferred_locale = partial.preferred_locale ?: preferred_locale,
            verification_level = partial.verification_level ?: verification_level,
            explicit_content_filter = partial.explicit_content_filter ?: explicit_content_filter
        )
    }
}
