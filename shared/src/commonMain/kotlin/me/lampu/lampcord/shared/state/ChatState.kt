package me.lampu.lampcord.shared.state

import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Permission
import me.lampu.lampcord.shared.utils.PermissionHelper
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.collectLatest
import kotlinx.serialization.json.*
import kotlin.random.Random

class ChatState(
    private val gatewayManager: GatewayManager,
    private val discordClient: DiscordClient,
    private val json: Json,
    val readStateStore: ReadStateStore,
    val presenceStore: PresenceStore,
    val userStore: UserStore,
    val guildStore: GuildStore,
    val memberListStore: MemberListStore,
    val messageStore: MessageStore,
    val tokenStore: TokenStore,
    val settingsStore: SettingsStore
) {
    val draftMessages = mutableStateMapOf<String, String>()

    var isConnected by mutableStateOf(false)
    var isConnecting by mutableStateOf(false)

    val typingUsers = mutableStateMapOf<String, MutableMap<String, Long>>()

    val availableCommands = mutableStateListOf<ApplicationCommand>()
    val availableApplications = mutableStateListOf<Application>()

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
    
    val relationships get() = userStore.relationships
    val presences get() = presenceStore.presences
    
    var currentUser by userStore::currentUser
    var userSettings by settingsStore::userSettings
    
    var selectedGuild by guildStore::selectedGuild
    var selectedChannel by guildStore::selectedChannel
    var selectedThread by guildStore::selectedThread
    
    val currentMember by derivedStateOf {
        val user = currentUser ?: return@derivedStateOf null
        selectedGuild?.let { userStore.getMember(it.id, user.id) } ?: selectedGuild?.members?.find { it.user?.id == user.id || (it.user == null && it.joined_at.isNotEmpty()) }
    }

    var selectedUser by mutableStateOf<User?>(null)
    var selectedProfile by mutableStateOf<UserProfile?>(null)
    var profilePosition by mutableStateOf<Offset?>(null)
    var replyingTo by mutableStateOf<Message?>(null)
    var editingMessage by mutableStateOf<Message?>(null)
    var forwardingMessage by mutableStateOf<Message?>(null)
    var isSettingsVisible by mutableStateOf(false)
    var isQuickSwitcherVisible by mutableStateOf(false)
    val pendingFiles = mutableStateListOf<Pair<String, ByteArray>>()
    
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
    private val activeMemberListChannels = mutableMapOf<String, MutableMap<String, List<List<Int>>>>()
    private var lastRequestedRanges = emptyList<List<Int>>()

    init {
        val savedToken = Settings.shared.discordToken
        if (savedToken.isNotBlank()) {
            connect(savedToken)
        }

        scope.launch {
            gatewayManager.events.collectLatest { payload ->
                handleGatewayEvent(payload)
            }
        }
    }

    private fun handleGatewayEvent(payload: GatewayPayload) {
        when (payload.t) {
            "READY" -> {
                isConnected = true
                isConnecting = false
                payload.d?.let { data ->
                    try {
                        val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                        currentUser = ready.user
                        currentToken?.let { tokenStore.addAccount(it, ready.user) }
                        userSettings = (ready.user_settings as? JsonObject)?.let { el ->
                            try {
                                json.decodeFromJsonElement<UserSettings>(el)
                            } catch (e: Exception) {
                                null
                            }
                        }
                        
                        guilds.clear()
                        guilds.addAll(ready.guilds)
                        
                        readStateStore.handleReady(ready)
                        
                        ready.guilds.forEach { guild ->
                            guild.members?.forEach { member ->
                                member.user?.let { user -> 
                                    userStore.cacheMember(guild.id, user.id, member)
                                    userStore.cacheUser(user)
                                }
                            }
                        }
                        
                        privateChannels.clear()
                        privateChannels.addAll(ready.private_channels.sortedByDescending { it.lastMessageId() ?: "0" })

                        scope.launch {
                            val friends = discordClient.getRelationships()
                            relationships.clear()
                            relationships.addAll(friends)
                        }
                        
                        applyGuildOrdering()
                        
                        // Background fetch all channels for forwarding/switcher cache
                        scope.launch {
                            ready.guilds.forEach { guild ->
                                if (guildStore.allGuildChannels[guild.id] == null) {
                                    try {
                                        val gChannels = discordClient.getGuildChannels(guild.id)
                                        if (gChannels.isNotEmpty()) {
                                            guildStore.allGuildChannels[guild.id] = gChannels.filter { it.type in listOf(0, 5, 4, 15) }.sortedBy { it.position }
                                        }
                                        kotlinx.coroutines.delay(500) // Increase delay to avoid rate limits
                                    } catch (e: Exception) { }
                                }
                            }
                        }

                        if (selectedGuild == null && selectedChannel == null) {
                            selectHome()
                        }
                    } catch (e: Exception) {
                        println("Error decoding READY: ${e.message}")
                    }
                }
            }
            "GUILD_CREATE" -> {
                payload.d?.let { data ->
                    try {
                        val guild = json.decodeFromJsonElement<Guild>(data)
                        guildStore.handleGuildCreate(guild)
                        applyGuildOrdering()
                    } catch (e: Exception) { }
                }
            }
            "MESSAGE_CREATE" -> {
                payload.d?.let { data ->
                    try {
                        val message = json.decodeFromJsonElement<Message>(data)
                        
                        message.guild_id?.let { guildId ->
                            message.member?.let { member ->
                                userStore.cacheMember(guildId, message.author.id, member)
                            }
                        }
                        userStore.cacheUser(message.author)
                        
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
                            val myId = currentUser?.id
                            if (myId != null && isMessageMentioningMe(message)) {
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
            "GUILD_MEMBER_LIST_UPDATE" -> {
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
            "THREAD_LIST_SYNC" -> {
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
            "THREAD_CREATE", "THREAD_UPDATE" -> {
                payload.d?.let { data ->
                    try {
                        val thread = json.decodeFromJsonElement<Channel>(data)
                        upsertForumThread(thread)
                    } catch (e: Exception) { }
                }
            }
            "THREAD_DELETE" -> {
                payload.d?.let { data ->
                    try {
                        val delete = json.decodeFromJsonElement<ThreadDeleteEvent>(data)
                        val forum = selectedChannel ?: return
                        if (forum.type == 15 && delete.parent_id == forum.id) {
                            forumThreads.removeAll { it.id == delete.id }
                            if (selectedThread?.id == delete.id) selectedThread = null
                        }
                    } catch (e: Exception) { }
                }
            }
            "MESSAGE_UPDATE" -> {
                payload.d?.let { data ->
                    try {
                        val partialMessage = json.decodeFromJsonElement<Message>(data)
                        messageStore.handleMessageUpdate(partialMessage, data.jsonObject)
                    } catch (e: Exception) { }
                }
            }
            "MESSAGE_DELETE" -> {
                payload.d?.let { data ->
                    try {
                        val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return@let
                        messageStore.handleMessageDelete(id)
                    } catch (e: Exception) { }
                }
            }
            "RELATIONSHIP_ADD" -> {
                payload.d?.let { data ->
                    try {
                        userStore.handleRelationshipAdd(json.decodeFromJsonElement<Relationship>(data))
                    } catch (e: Exception) { }
                }
            }
            "RELATIONSHIP_REMOVE" -> {
                payload.d?.let { data ->
                    try {
                        val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return@let
                        userStore.handleRelationshipRemove(id)
                    } catch (e: Exception) { }
                }
            }
            "USER_SETTINGS_UPDATE" -> {
                payload.d?.let { data ->
                    try {
                        val newSettings = json.decodeFromJsonElement<UserSettings>(data)
                        settingsStore.handleUserSettingsUpdate(newSettings)
                    } catch (e: Exception) { }
                }
            }
            "PRESENCE_UPDATE" -> {
                payload.d?.let { data ->
                    try {
                        presenceStore.handlePresenceUpdate(json.decodeFromJsonElement<PresenceUpdate>(data))
                    } catch (e: Exception) { }
                }
            }
            "MESSAGE_ACK" -> {
                payload.d?.let { data ->
                    try {
                        readStateStore.handleMessageAck(json.decodeFromJsonElement<MessageAcknowledge>(data))
                    } catch (e: Exception) { }
                }
            }
            "TYPING_START" -> {
                payload.d?.let { data ->
                    try {
                        val typing = json.decodeFromJsonElement<TypingStart>(data)
                        if (typing.user_id != currentUser?.id) {
                            val channelTyping = typingUsers.getOrPut(typing.channel_id) { mutableStateMapOf() }
                            channelTyping[typing.user_id] = getCurrentTimeMillis()
                            
                            // Remove after 10 seconds
                            scope.launch {
                                kotlinx.coroutines.delay(10000L)
                                if (channelTyping[typing.user_id] != null) {
                                     channelTyping.remove(typing.user_id)
                                }
                            }
                        }
                    } catch (e: Exception) { }
                }
            }
            // Reactions could also be moved to a ReactionStore if needed
            "MESSAGE_REACTION_ADD" -> handleReactionAddEvent(payload)
            "MESSAGE_REACTION_REMOVE" -> handleReactionRemoveEvent(payload)
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
                        if (newCount == 0) reactions.removeAt(emojiIndex)
                        else reactions[emojiIndex] = r.copy(count = newCount, me = if (isMe) false else r.me)
                        messages[index] = msg.copy(reactions = reactions)
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private val selectedMemberListId: String?
        get() {
            val guild = selectedGuild ?: return null
            val channel = selectedThread ?: selectedChannel ?: return null
            return channel.memberListId(guild)
        }

    private suspend fun loadForumThreads(channelId: String): List<Channel> {
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
        return all.distinctBy { it.id }.sortedByDescending { it.forumSortKey() }
    }

    private fun upsertForumThread(thread: Channel) {
        val forum = selectedChannel ?: return
        if (forum.type != 15 || thread.parent_id != forum.id) return
        val index = forumThreads.indexOfFirst { it.id == thread.id }
        if (index == -1) forumThreads.add(thread) else forumThreads[index] = thread
        val sorted = forumThreads.sortedByDescending { it.forumSortKey() }
        forumThreads.clear()
        forumThreads.addAll(sorted)
    }

    private fun Channel.forumSortKey(): Long = lastMessageId()?.toLongOrNull() ?: id.toLongOrNull() ?: 0L

    private fun applyGuildOrdering() {
        val order = userSettings?.guild_positions ?: return
        if (order.isEmpty()) return
        val sorted = guilds.sortedBy { guild -> val pos = order.indexOf(guild.id); if (pos == -1) Int.MAX_VALUE else pos }
        guildStore.guilds.clear()
        guildStore.guilds.addAll(sorted)
    }

    fun connect(token: String) {
        if (isConnected) {
            disconnect()
        }
        isConnecting = true
        isConnected = false
        currentToken = token
        subscribedGuilds.clear()
        activeMemberListChannels.clear()
        Settings.shared.discordToken = token
        discordClient.setToken(token)
        gatewayManager.connect(token)
    }

    fun disconnect() {
        gatewayManager.disconnect()
        isConnected = false
        isConnecting = false
        clearAllStores()
    }

    private fun clearAllStores() {
        guildStore.guilds.clear()
        guildStore.channels.clear()
        guildStore.privateChannels.clear()
        guildStore.forumThreads.clear()
        guildStore.selectedGuild = null
        guildStore.selectedChannel = null
        guildStore.selectedThread = null
        
        messageStore.clear()
        memberListStore.clear()
        readStateStore.readStates.clear()
        presenceStore.presences.clear()
        userStore.relationships.clear()
        userStore.currentUser = null
        settingsStore.userSettings = null
    }

    suspend fun login(login: String, password: String) = discordClient.login(LoginRequest(login, password), discordClient.getFingerprint() ?: "").also { 
        if (it?.token != null) {
            connect(it.token) 
        }
    }

    suspend fun verifyMFA(code: String, ticket: String, type: String = "totp"): Boolean {
        val response = discordClient.loginMFA(MFALoginRequest(code, ticket), currentFingerprint ?: "", type)
        if (response?.token != null) {
            connect(response.token)
            return true
        }
        return false
    }

    fun selectGuild(guild: Guild) {
        if (selectedGuild?.id == guild.id) return
        guildLoadingJob?.cancel()
        selectedGuild = guild
        channels.clear()
        memberListStore.clear()
        lastRequestedRanges = emptyList()
        guildLoadingJob = scope.launch {
            subscribeToGuild(guild.id)
            val guildChannels = discordClient.getGuildChannels(guild.id)
            if (guildChannels.isNotEmpty()) {
                val filtered = guildChannels.filter { it.type in listOf(0, 5, 4, 15) }.sortedBy { it.position }
                guildStore.allGuildChannels[guild.id] = filtered
                channels.clear()
                channels.addAll(filtered)
            }
            val lastChannelId = Settings.shared.getLastChannel(guild.id)
            val channelToSelect = if (lastChannelId != null) channels.find { it.id == lastChannelId } else channels.firstOrNull { it.type in listOf(0, 5, 15) }
            channelToSelect?.let { selectChannel(it) }
            
            // Fetch commands
            try {
                val index = discordClient.getCommandIndex(guild.id)
                availableCommands.clear()
                availableApplications.clear()
                index?.let {
                    availableCommands.addAll(it.application_commands)
                    availableApplications.addAll(it.applications)
                }
            } catch (e: Exception) { }
        }
    }

    fun isUnread(channel: Channel) = readStateStore.isUnread(channel)
    fun getMentionCount(channelId: String) = readStateStore.getMentionCount(channelId)

    fun selectHome() {
        selectedGuild = null
        memberListStore.clear()
        lastRequestedRanges = emptyList()
        val lastDmId = Settings.shared.getLastChannel("home")
        val dmToSelect = if (lastDmId != null) privateChannels.find { it.id == lastDmId } else privateChannels.firstOrNull()
        dmToSelect?.let { selectChannel(it) }
    }

    fun selectChannel(channel: Channel) {
        if (selectedChannel?.id == channel.id) return
        channelLoadingJob?.cancel()
        selectedChannel = channel
        selectedThread = null
        memberListStore.clear()
        lastRequestedRanges = emptyList()
        Settings.shared.setLastChannel(selectedGuild?.id ?: "home", channel.id)
        channelLoadingJob = scope.launch {
            if (channel.type == 15) {
                isForumLoading = true
                try { forumThreads.clear(); forumThreads.addAll(loadForumThreads(channel.id)) } finally { isForumLoading = false }
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

    fun selectThread(thread: Channel) {
        selectedThread = thread
        messageStore.clear()
        lastRequestedRanges = emptyList()
        memberListStore.clear()
        scope.launch {
            val channelMessages = discordClient.getChannelMessages(thread.id)
            channelMessages.forEach { msg -> msg.member?.let { m -> userStore.cacheMember(selectedGuild?.id ?: "", msg.author.id, m) } }
            messageStore.addMessages(channelMessages)
            requestMemberListRange(listOf(listOf(0, 99)))
        }
    }

    fun loadMoreMessages() {
        val channelId = (selectedThread ?: selectedChannel)?.id ?: return
        if (messageStore.isLoadingHistory || !messageStore.hasMoreHistory) return
        val oldestMessage = messages.lastOrNull() ?: return
        messageStore.isLoadingHistory = true
        historyLoadingJob = scope.launch {
            val moreMessages = discordClient.getChannelMessages(channelId, before = oldestMessage.id)
            if (moreMessages.isEmpty()) messageStore.hasMoreHistory = false
            else {
                val newMessages = moreMessages.filter { msg -> messages.none { it.id == msg.id } }
                if (newMessages.isEmpty()) messageStore.hasMoreHistory = false
                else {
                    newMessages.forEach { msg -> msg.member?.let { m -> userStore.cacheMember(selectedGuild?.id ?: "", msg.author.id, m) } }
                    messageStore.addMessages(newMessages)
                }
            }
            messageStore.isLoadingHistory = false
        }
    }

    fun sendMessage(content: String) {
        val channelId = (selectedThread ?: selectedChannel)?.id ?: return
        if (editingMessage != null) {
            scope.launch { if (discordClient.editMessage(channelId, editingMessage!!.id, content)) editingMessage = null }
            return
        }
        val user = currentUser ?: return
        messageStore.sendMessage(channelId, content, user, replyingTo?.id, pendingFiles.toList(), selectedGuild?.id)
        replyingTo = null
        pendingFiles.clear()
    }

    fun forwardMessage(destinationChannel: Channel, message: Message) {
        val user = currentUser ?: return
        messageStore.sendMessage(
            destinationChannel.id, 
            "", 
            user, 
            null, 
            emptyList(), 
            destinationChannel.guild_id,
            forwardFrom = message
        )
        forwardingMessage = null
    }

    fun retryMessage(message: Message) = messageStore.retryMessage(message)
    fun deletePendingMessage(message: Message) = messageStore.deletePendingMessage(message)

    fun showProfile(userId: String, position: Offset? = null) {
        selectedProfile = null
        profilePosition = position
        scope.launch { selectedProfile = discordClient.getUserProfile(userId, selectedGuild?.id) }
    }

    fun getUserStatus(userId: String) = presenceStore.getUserStatus(userId, currentUser?.id, userSettings?.status)
    fun updateStatus(status: String) = scope.launch { if (presenceStore.updateStatus(status)) userSettings = userSettings?.copy(status = status) }
    fun updateCustomStatus(text: String?) = scope.launch { if (presenceStore.updateCustomStatus(text)) userSettings = userSettings?.copy(custom_status = me.lampu.lampcord.shared.model.CustomStatus(text = text)) }

    fun leaveGuild(guildId: String) = scope.launch {
        if (discordClient.leaveGuild(guildId)) {
            guildStore.guilds.removeAll { it.id == guildId }
            if (selectedGuild?.id == guildId) {
                selectHome()
            }
        }
    }

    fun markGuildAsRead(guildId: String) = scope.launch {
        val channelIds = channels.filter { it.guild_id == guildId }.map { it.id }
        if (discordClient.ackBulk(channelIds)) {
            channelIds.forEach { id ->
                readStateStore.readStates[id]?.let {
                    readStateStore.readStates[id] = it.copy(mention_count = 0)
                }
            }
        }
    }

    fun markCategoryAsRead(categoryId: String) = scope.launch {
        val channelIds = channels.filter { it.parent_id == categoryId }.map { it.id }
        if (discordClient.ackBulk(channelIds)) {
            channelIds.forEach { id ->
                readStateStore.readStates[id]?.let {
                    readStateStore.readStates[id] = it.copy(mention_count = 0)
                }
            }
        }
    }

    fun getMember(guildId: String, userId: String) = userStore.getMember(guildId, userId)

    fun sendInteraction(command: ApplicationCommand) {
        val guildId = selectedGuild?.id ?: return
        val channelId = (selectedThread ?: selectedChannel)?.id ?: return
        val sessionId = gatewayManager.sessionId ?: return
        
        scope.launch {
            val request = InteractionRequest(
                type = 2,
                application_id = command.application_id,
                guild_id = guildId,
                channel_id = channelId,
                session_id = sessionId,
                data = InteractionData(
                    id = command.id,
                    name = command.name,
                    version = command.version
                ),
                nonce = "${getCurrentTimeMillis()}${Random.nextInt(1000, 9999)}"
            )
            discordClient.sendInteraction(request)
        }
    }

    fun removeReaction(channelId: String, messageId: String, emoji: String) = scope.launch { discordClient.removeReaction(channelId, messageId, emoji) }
    fun addReaction(channelId: String, messageId: String, emoji: String) = scope.launch { discordClient.addReaction(channelId, messageId, emoji) }

    fun isMessageMentioningMe(message: Message): Boolean {
        if (message.mention_everyone) return true
        val myId = currentUser?.id ?: return false
        if (message.content.contains("<@$myId>") || message.content.contains("<@!$myId>")) return true
        selectedGuild?.id?.let { guildId -> getMember(guildId, myId)?.roles?.forEach { roleId -> if (message.content.contains("<@&$roleId>")) return true } }
        return false
    }

    fun hasPermission(permission: Permission): Boolean {
        val guild = selectedGuild ?: return true
        val member = currentMember ?: return true
        return PermissionHelper.hasPermission(member, guild, selectedThread ?: selectedChannel, permission)
    }

    fun requestMemberListRange(ranges: List<List<Int>>) {
        if (ranges == lastRequestedRanges) return
        val guildId = selectedGuild?.id ?: return
        val channelId = (selectedThread ?: selectedChannel)?.id ?: return
        lastRequestedRanges = ranges
        scope.launch {
            subscribeToGuild(guildId)
            val guildChannels = activeMemberListChannels.getOrPut(guildId) { mutableMapOf() }
            guildChannels.clear()
            guildChannels[channelId] = ranges
            val update = GuildSubscriptionsUpdate(subscriptions = mapOf(guildId to GuildSubscription(channels = guildChannels)))
            gatewayManager.sendPayload(GatewayPayload(op = 37, d = json.encodeToJsonElement(update)))
        }
    }

    private suspend fun subscribeToGuild(guildId: String) {
        if (guildId in subscribedGuilds) return
        subscribedGuilds.add(guildId)
        val d = buildJsonObject { put("subscriptions", buildJsonObject { put(guildId, buildJsonObject { put("typing", true); put("threads", true); put("activities", true); put("member_updates", true); put("channels", buildJsonObject { }) }) }) }
        gatewayManager.sendPayload(GatewayPayload(op = 37, d = d))
    }
}
