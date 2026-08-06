package me.lampu.lampcord.shared.state

import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.geometry.Offset
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
    val pendingFiles = mutableStateListOf<Pair<String, ByteArray>>()

    var currentVoiceState by mutableStateOf<VoiceState?>(null)
    var isVoiceConnected by mutableStateOf(false)
    var voiceConnectionDuration by mutableStateOf(0L)
    private var voiceTimerJob: Job? = null
    val voiceStates = mutableStateMapOf<String, SnapshotStateMap<String, VoiceState>>() // guildId -> userId -> VoiceState

    var isVoiceChatTextVisible by mutableStateOf(false)
    var isChannelsAndRolesVisible by mutableStateOf(false)
    var isServerSettingsVisible by mutableStateOf(false)

    var activeCommand by mutableStateOf<ApplicationCommand?>(null)
    val commandOptions = mutableStateMapOf<String, JsonElement>()

    // Attachment viewer (matches Paicord's attachmentViewerAttachments/Index/Visible)
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

    private val subscribedGuilds = mutableSetOf<String>()
    private var lastRequestedRanges = emptyList<List<Int>>()

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
            "GUILD_CREATE" -> handleGuildCreate(payload)
            "GUILD_DELETE" -> handleGuildDelete(payload)
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
            val user = try {
                data.jsonObject["user"]?.let { json.decodeFromJsonElement<User>(it) }
            } catch (e: Exception) {
                println("Failed to decode user from READY: ${e.message}")
                null
            }

            user?.let { u ->
                currentUser = u
                currentToken?.let { t ->
                    tokenStore.addAccount(t, u)
                    Settings.shared.discordToken = t
                }
            }

            try {
                val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                
                (ready.user_settings as? JsonObject)?.let { el ->
                    try {
                        val settings = json.decodeFromJsonElement<UserSettings>(el)
                        settingsStore.userSettings = settings
                    } catch (e: Exception) { }
                }
                
                userGuildSettingsStore.handleReady(ready)
                guildStore.setGuilds(ready.guilds, settingsStore.userSettings?.guild_positions ?: emptyList())
                guildStore.setPrivateChannels(ready.private_channels.sortedByDescending { it.lastMessageId() ?: "0" })
                
                readStateStore.handleReady(ready)
                
                ready.guilds.forEach { guild ->
                    guild.members?.forEach { member ->
                        member.user?.let { user -> 
                            userStore.cacheMember(guild.id, user.id, member)
                            userStore.cacheUser(user)
                        }
                    }
                }

                relationshipStore.fetchRelationships()
                
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

                // Matches Paicord's initial voice state update
                scope.launch {
                    delay(500)
                    gatewayManager.sendVoiceStateUpdate(
                        guildId = null,
                        channelId = null,
                        selfMute = true,
                        selfDeaf = true,
                        selfVideo = false
                    )
                }

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
                guildStore.handleGuildCreate(guild, userSettings?.guild_positions ?: emptyList())
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

    private fun handleMessageCreate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val message = json.decodeFromJsonElement<Message>(data)
                
                message.guild_id?.let { guildId ->
                    message.member?.let { member ->
                        userStore.cacheMember(guildId, message.author.id, member)
                    }
                }
                userStore.cacheUser(message.author)
                
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
                if (update.guild_id == selectedGuild?.id && update.id == selectedMemberListId) {
                    memberListStore.handleMemberListUpdate(update)
                    update.ops.forEach { op ->
                        op.items?.forEach { it.member?.let { m -> m.user?.let { u -> 
                            userStore.cacheMember(update.guild_id, u.id, m)
                            userStore.cacheUser(u)
                            m.presence?.let { p -> presenceStore.handlePresenceUpdate(p) }
                        } } }
                        op.item?.member?.let { m -> m.user?.let { u -> 
                            userStore.cacheMember(update.guild_id, u.id, m)
                            userStore.cacheUser(u)
                            m.presence?.let { p -> presenceStore.handlePresenceUpdate(p) }
                        } }
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
                guildStore.setGuilds(guilds, newSettings.guild_positions)
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

    private val selectedMemberListId: String?
        get() {
            val guild = selectedGuild ?: return null
            val channel = selectedChannel ?: return null
            return channel.memberListId(guild)
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
        userStore.currentUser = null
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
        lastRequestedRanges = emptyList()
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
        return folder.guild_ids.any { isGuildUnread(it) }
    }
    
    fun getGuildMentionCount(guildId: String): Int {
        return guildStore.allGuildChannels[guildId]?.sumOf { getMentionCount(it.id) } ?: 0
    }
    
    fun getFolderMentionCount(folder: GuildFolder): Int {
        return folder.guild_ids.sumOf { getGuildMentionCount(it) }
    }

    fun getMentionCount(channelId: String) = readStateStore.getMentionCount(channelId)

    fun selectHome() {
        selectedGuild = null
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        memberListStore.clear()
        lastRequestedRanges = emptyList()
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
        memberListStore.clear()
        lastRequestedRanges = emptyList()
        Settings.shared.setLastChannel(selectedGuild?.id ?: "home", channel.id)
        
        if (channel.type == 2 || channel.type == 13) {
            connectToVoice(channel)
        }

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
                messageStore.clear()
                val channelMessages = discordClient.getChannelMessages(channel.id)
                channelMessages.forEach { msg -> msg.member?.let { m -> userStore.cacheMember(channel.guild_id ?: selectedGuild?.id ?: "", msg.author.id, m) } }
                messageStore.addMessages(channelMessages)
                channelMessages.firstOrNull()?.let { readStateStore.ackMessage(channel.id, it.id) }
            }
            if (channel.guild_id != null || selectedGuild != null) requestMemberListRange(listOf(listOf(0, 99)))
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
            channelMessages.forEach { msg -> msg.member?.let { m -> userStore.cacheMember(channel.guild_id ?: selectedGuild?.id ?: "", msg.author.id, m) } }
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
                    more.forEach { msg -> msg.member?.let { m -> userStore.cacheMember(selectedGuild?.id ?: "", msg.author.id, m) } }
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
        return PermissionHelper.hasPermission(member, guild, channel, permission)
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

    private fun subscribeToGuild(guildId: String) {
        if (guildId in subscribedGuilds) return
        subscribedGuilds.add(guildId)
        gatewayManager.sendSubscription(guildId)
    }

    fun requestMemberListRange(ranges: List<List<Int>>) {
        val guildId = selectedGuild?.id ?: return
        val channelId = selectedChannel?.id ?: return
        
        if (ranges == lastRequestedRanges) return
        lastRequestedRanges = ranges

        gatewayManager.sendLazyRequest(guildId, channelId, ranges)
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
        if (message.author.id == myId) return false
        
        if (message.mentions.any { it.id == myId }) return true
        
        val myMember = currentMember
        if (myMember != null) {
            if (message.mention_roles.any { roleId -> roleId in myMember.roles }) return true
        }
        
        if (message.mention_everyone) return true
        
        return false
    }
}
