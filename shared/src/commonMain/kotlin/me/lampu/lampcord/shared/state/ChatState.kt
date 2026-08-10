@file:OptIn(ExperimentalResourceApi::class)

package me.lampu.lampcord.shared.state

import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.gateway.VoiceGatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.*
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import kotlin.random.Random
import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.hours
import kotlin.time.Clock
import kotlin.time.Instant
import lampcord.shared.generated.resources.Res
import org.jetbrains.compose.resources.ExperimentalResourceApi

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
    val voiceStore: VoiceStore,
    val searchStore: SearchStore,
    val profileStore: ProfileStore,
    val autocompleteStore: AutocompleteStore,
    val navigationStore: NavigationStore,
    val gatewayHandler: GatewayHandler,
    val commandStore: CommandStore,
    val experimentStore: ExperimentStore,
    val tokenStore: TokenStore,
    val settingsStore: SettingsStore,
    val errorStore: AppErrorStore
) {
    val draftMessages = mutableStateMapOf<String, String>()

    var isConnected
        get() = navigationStore.isConnected
        set(value) { navigationStore.isConnected = value }
    var isConnecting
        get() = navigationStore.isConnecting
        set(value) { navigationStore.isConnecting = value }

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
    val onlineCount get() = memberListStore.onlineCount
    val totalMemberCount get() = memberListStore.memberCount
    
    val relationships get() = relationshipStore.relationships
    val presences get() = presenceStore.presences
    
    var currentUser by userStore::currentUser
    var userSettings by settingsStore::userSettings
    
    var selectedGuild by navigationStore::selectedGuild
    var selectedChannel by navigationStore::selectedChannel
    var selectedThread by navigationStore::selectedThread
    
    var selectedGuildOnboarding
        get() = navigationStore.selectedGuildOnboarding
        set(value) { navigationStore.selectedGuildOnboarding = value }
    
    var isFriendsSelected
        get() = navigationStore.isFriendsSelected
        set(value) { navigationStore.isFriendsSelected = value }
    
    val currentMember by derivedStateOf {
        val user = currentUser ?: return@derivedStateOf null
        selectedGuild?.let { userStore.getMember(it.id, user.id) } ?: selectedGuild?.members?.find { it.user?.id == user.id || (it.user == null && it.joined_at.isNotEmpty()) }
    }

    var selectedUser by mutableStateOf<User?>(null)
    
    var selectedProfile
        get() = profileStore.selectedProfile
        set(value) { profileStore.selectedProfile = value }
    var sidebarProfile
        get() = profileStore.sidebarProfile
        set(value) { profileStore.sidebarProfile = value }
    var isSidebarProfileLoading get() = profileStore.isSidebarProfileLoading
        set(value) { profileStore.isSidebarProfileLoading = value }
    var isProfileExpanded get() = profileStore.isProfileExpanded
        set(value) { profileStore.isProfileExpanded = value }
    var isProfileLoading get() = profileStore.isProfileLoading
        set(value) { profileStore.isProfileLoading = value }
    var profilePosition get() = profileStore.profilePosition
        set(value) { profileStore.profilePosition = value }
    var replyingTo by mutableStateOf<Message?>(null)
    var editingMessage by mutableStateOf<Message?>(null)
    var forwardingMessage by mutableStateOf<Message?>(null)
    
    var isSettingsVisible
        get() = navigationStore.isSettingsVisible
        set(value) { navigationStore.isSettingsVisible = value }
    var isQuickSwitcherVisible
        get() = navigationStore.isQuickSwitcherVisible
        set(value) { navigationStore.isQuickSwitcherVisible = value }
    
    var scrollToMessageId by mutableStateOf<String?>(null)
    var highlightedMessageId by mutableStateOf<String?>(null)
    val pendingFiles = mutableStateListOf<PendingFile>()

    val currentVoiceState get() = voiceStore.currentVoiceState
    val isVoiceConnected get() = voiceStore.isVoiceConnected
    val voiceConnectionDuration get() = voiceStore.voiceConnectionDuration
    val voiceStates get() = voiceStore.voiceStates

    var isVoiceChatTextVisible
        get() = voiceStore.isVoiceChatTextVisible
        set(value) { voiceStore.isVoiceChatTextVisible = value }

    var isChannelsAndRolesVisible
        get() = navigationStore.isChannelsAndRolesVisible
        set(value) { navigationStore.isChannelsAndRolesVisible = value }
    var isServerMenuVisible
        get() = navigationStore.isServerMenuVisible
        set(value) { navigationStore.isServerMenuVisible = value }
    var isServerSettingsVisible
        get() = navigationStore.isServerSettingsVisible
        set(value) { navigationStore.isServerSettingsVisible = value }
    var isMediaPickerVisible
        get() = navigationStore.isMediaPickerVisible
        set(value) { navigationStore.isMediaPickerVisible = value }
    var isPinsVisible
        get() = navigationStore.isPinsVisible
        set(value) { navigationStore.isPinsVisible = value }
    val pinnedMessages = mutableStateListOf<Message>()

    var isSearchVisible
        get() = navigationStore.isSearchVisible
        set(value) { navigationStore.isSearchVisible = value }
    var searchQuery
        get() = searchStore.searchQuery
        set(value) { searchStore.searchQuery = value }
    val searchResults get() = searchStore.searchResults
    var isSearchLoading get() = searchStore.isSearchLoading
        set(value) { searchStore.isSearchLoading = value }
    var totalSearchResults get() = searchStore.totalSearchResults
        set(value) { searchStore.totalSearchResults = value }
    val searchHistory get() = searchStore.searchHistory

    val loadingMessages = mutableStateListOf<String>()

    var activeCommand by mutableStateOf<ApplicationCommand?>(null)
    val commandOptions = mutableStateMapOf<String, JsonElement>()

    var autocompleteType
        get() = autocompleteStore.autocompleteType
        set(value) { autocompleteStore.autocompleteType = value }
    var autocompleteQuery
        get() = autocompleteStore.autocompleteQuery
        set(value) { autocompleteStore.autocompleteQuery = value }
    var autocompleteSelectedIndex
        get() = autocompleteStore.autocompleteSelectedIndex
        set(value) { autocompleteStore.autocompleteSelectedIndex = value }
    val autocompleteItems get() = autocompleteStore.autocompleteItems

    fun updateAutocomplete(type: AutocompleteType?, query: String) = autocompleteStore.updateAutocomplete(type, query, selectedGuild)

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

    var isLoadingHistory by messageStore::isLoadingHistory
    var hasMoreHistory by messageStore::hasMoreHistory

    var isForumLoading
        get() = navigationStore.isForumLoading
        set(value) { navigationStore.isForumLoading = value }

    private val scope = CoroutineScope(Dispatchers.Main)

    init {
        scope.launch {
            me.lampu.lampcord.shared.utils.EmojiIndex.initialize()
        }

        scope.launch {
            try {
                val text = ResourceLoader.readText("files/loading_messages.txt")
                val lines = text?.split("\n")
                    ?.map { it.trim() }
                    ?.filter { it.isNotEmpty() && !it.startsWith("#") }
                if (lines != null && lines.isNotEmpty()) {
                    loadingMessages.addAll(lines)
                }
            } catch (e: Exception) {
                loadingMessages.add("how")
            }
        }

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

    private val subscribedGuilds = mutableSetOf<String>()

    private fun handleGatewayEvent(payload: GatewayPayload) {
        gatewayHandler.handleGatewayEvent(payload)
    }

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

    fun selectHome() = navigationStore.selectHome()
    fun selectFriends() = navigationStore.selectFriends()
    fun selectGuild(guild: Guild) = navigationStore.selectGuild(guild) { subscribeToGuild(it) }
    fun selectChannel(channel: Channel) {
        navigationStore.selectChannel(channel)
        isVoiceChatTextVisible = false
        if (channel.guild_id != null || selectedGuild != null) requestMemberListRange(listOf(listOf(0, 99)))
    }
    fun selectThread(channel: Channel) {
        navigationStore.selectThread(channel)
        isVoiceChatTextVisible = false
    }

    fun isUnread(channel: Channel): Boolean {
        if (channel.type == 4 || channel.type == 2 || channel.type == 13) return false
        if (userGuildSettingsStore.isChannelMuted(channel.guild_id, channel.id)) return false
        channel.parent_id?.let { if (userGuildSettingsStore.isChannelMuted(channel.guild_id, it)) return false }
        return readStateStore.isUnread(channel)
    }

    fun isGuildUnread(guildId: String): Boolean {
        if (userGuildSettingsStore.isGuildMuted(guildId)) return false
        return guildStore.allGuildChannels[guildId]?.any { isUnread(it) } ?: false
    }

    fun isFolderUnread(folder: GuildFolder): Boolean {
        return folder.guild_ids.any { el -> 
            val id = el.jsonPrimitive.contentOrNull ?: return@any false
            isGuildUnread(id) 
        }
    }
    
    fun getGuildMentionCount(guildId: String): Int {
        return guildStore.allGuildChannels[guildId]?.sumOf { getMentionCount(it.id) } ?: 0
    }
    
    fun getFolderMentionCount(folder: GuildFolder): Int {
        return folder.guild_ids.sumOf { el -> 
            val id = el.jsonPrimitive.contentOrNull ?: return@sumOf 0
            getGuildMentionCount(id)
        }
    }
    
    fun getMentionCount(channelId: String) = readStateStore.getMentionCount(channelId)

    fun connectToVoice(channel: Channel) = voiceStore.connectToVoice(channel)

    fun disconnectFromVoice() = voiceStore.disconnectFromVoice(selectedGuild?.id)

    fun toggleVoiceMute() = voiceStore.toggleVoiceMute()

    fun toggleVoiceDeaf() = voiceStore.toggleVoiceDeaf()

    fun toggleVoiceVideo() = voiceStore.toggleVoiceVideo()

    fun toggleVoiceStream() = voiceStore.toggleVoiceStream()

    fun loadMoreMessages() {
        val channel = selectedThread ?: selectedChannel ?: return
        messageStore.loadMoreMessages(channel.id, selectedGuild?.id, selectedThread?.id)
    }

    fun performSearch() = searchStore.performSearch(selectedGuild, selectedChannel)

    fun sendMessage(content: String, poll: Poll? = null) {
        val channel = selectedThread ?: selectedChannel ?: return

        if (editingMessage != null) {
            editMessage(editingMessage!!, content)
            editingMessage = null
            draftMessages.remove(channel.id)
            return
        }

        messageStore.sendMessage(
            channelId = channel.id,
            content = content,
            currentUser = currentUser!!,
            replyTo = replyingTo?.id,
            files = pendingFiles.toList(),
            guildId = selectedGuild?.id,
            forwardFrom = null,
            poll = poll
        )
        replyingTo = null
        pendingFiles.clear()
        
        // Clear draft
        draftMessages.remove(channel.id)
    }

    fun editMessage(message: Message, content: String) = messageStore.editMessage(message, content)

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

    fun deleteMessage(message: Message) = messageStore.deleteMessage(message)

    fun retryMessage(message: Message) = messageStore.retryMessage(message)
    fun deletePendingMessage(message: Message) = messageStore.deletePendingMessage(message)

    private var lastTypingTime = 0L

    fun sendTyping() {
        if (Settings.shared.silentTyping) return
        val channel = selectedChannel ?: return
        val now = getCurrentTimeMillis()
        if (now - lastTypingTime > 8000) {
            lastTypingTime = now
            scope.launch {
                discordClient.sendTyping(channel.id)
            }
        }
    }

    fun showProfile(userId: String, position: Offset? = null) = profileStore.showProfile(userId, selectedGuild?.id, position)

    fun getUserStatus(userId: String): String = presenceStore.getUserStatus(userId, currentUser?.id, userSettings?.status)

    fun isStatusVisible(user: User, presence: PresenceUpdate?, isStreaming: Boolean): Boolean {
        val flags = (user.public_flags ?: 0) or (user.flags ?: 0)
        // 524288 = PUBLIC_FLAG_BOT_HTTP_INTERACTIONS
        return if ((flags and 524288) != 0) {
            presence != null && presence.status != "offline" && presence.status != "invisible"
        } else {
            presence != null || isStreaming
        }
    }

    fun updateStatus(status: String) = scope.launch { discordClient.updateStatus(status) }
    fun updateCustomStatus(text: String?) = scope.launch { discordClient.updateCustomStatus(text) }

    fun leaveGuild(guildId: String) = guildStore.leaveGuild(guildId) {
        if (selectedGuild?.id == guildId) selectHome()
    }

    fun markGuildAsRead(guildId: String = selectedGuild?.id ?: "") = guildStore.markGuildAsRead(guildId)

    fun markCategoryAsRead(categoryId: String, guildId: String? = selectedGuild?.id) = guildStore.markCategoryAsRead(categoryId, guildId)

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

    fun pinMessage(message: Message) = messageStore.pinMessage(message)

    fun unpinMessage(message: Message) = messageStore.unpinMessage(message)

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
        if (requestKey == navigationStore.lastRequestedKey) return
        navigationStore.lastRequestedKey = requestKey
        gatewayManager.sendLazyRequest(guild.id, channel.id, ranges)
    }

    fun toggleMuteGuild(guildId: String) {
        val currentMuted = userGuildSettingsStore.isGuildMuted(guildId)
        if (currentMuted) {
            unmuteGuild(guildId)
        } else {
            muteGuild(guildId, Duration.INFINITE)
        }
    }

    fun unmuteGuild(guildId: String) {
        scope.launch {
            discordClient.updateUserGuildSettings(
                guildId,
                UserGuildSettings.Partial(
                    muted = false,
                    mute_config = null
                )
            )
        }
    }

    fun muteGuild(guildId: String, duration: Duration?) {
        // If duration is NOT null, we always mute for that duration.
        // If duration is INFINITE, we mute indefinitely.
        
        val muteConfig = if (duration != null && duration != Duration.INFINITE) {
            val endTime = (Clock.System.now() + duration).toString()
            MuteConfig(end_time = endTime)
        } else {
            MuteConfig(end_time = null)
        }

        scope.launch {
            discordClient.updateUserGuildSettings(
                guildId,
                UserGuildSettings.Partial(
                    muted = true,
                    mute_config = muteConfig
                )
            )
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

    fun setServerDMsAllowed(guildId: String, allowed: Boolean) {
        // This is a placeholder logic, usually this is a bitmask or a separate field in real Discord API
        // For now we use message_notifications as a proxy or just send a partial update if the model supports more
        scope.launch {
            discordClient.updateUserGuildSettings(
                guildId,
                UserGuildSettings.Partial(
                    message_notifications = if (allowed) 0 else 2
                )
            )
        }
    }

    fun setHideMutedChannels(guildId: String, hide: Boolean) {
        scope.launch {
            discordClient.updateUserGuildSettings(
                guildId,
                UserGuildSettings.Partial(
                    hide_muted_channels = hide
                )
            )
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

    fun updateUserSettings(partial: UserSettings.Partial) = settingsStore.updateUserSettings(partial)

    fun updateGuild(guildId: String, partial: Guild.Partial) = guildStore.updateGuild(guildId, partial)
}
