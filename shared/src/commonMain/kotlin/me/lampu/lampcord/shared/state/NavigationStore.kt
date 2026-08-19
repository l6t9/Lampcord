package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Onboarding
import me.lampu.lampcord.shared.model.memberListId
import me.lampu.lampcord.shared.notifications.MessageNotifier
import me.lampu.lampcord.shared.settings.Settings

class NavigationStore(
    private val channelApi: ChannelApi,
    private val guildApi: GuildApi,
    private val messageApi: MessageApi,
    private val userApi: UserApi,
    private val guildStore: GuildStore,
    private val memberListStore: MemberListStore,
    private val messageStore: MessageStore,
    private val userStore: UserStore,
    private val readStateStore: ReadStateStore,
    private val profileStore: ProfileStore,
    private val commandStore: CommandStore,
    private val selectionStore: SelectionStore,
    private val finderStore: FinderStore,
    private val notifier: MessageNotifier?,
    private val scope: CoroutineScope,
    val onChannelSelected: () -> Unit = {}
) {
    private val _focusChatRequest = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    val focusChatRequest = _focusChatRequest.asSharedFlow()

    private fun triggerFocusChat() {
        _focusChatRequest.tryEmit(Unit)
        onChannelSelected()
    }

    var selectedGuild: me.lampu.lampcord.shared.model.Guild?
        get() = selectionStore.selectedGuild
        set(value) { selectionStore.selectedGuild = value }

    var selectedChannel: me.lampu.lampcord.shared.model.Channel?
        get() = selectionStore.selectedChannel
        set(value) { selectionStore.selectedChannel = value }

    var selectedThread: me.lampu.lampcord.shared.model.Channel?
        get() = selectionStore.selectedThread
        set(value) { selectionStore.selectedThread = value }

    var selectedGuildOnboarding by mutableStateOf<Onboarding?>(null)
    var isFriendsSelected by mutableStateOf(false)
    var isMentionsSelected by mutableStateOf(false)

    var isConnected by mutableStateOf(false)
    var isConnecting by mutableStateOf(false)

    var isBubble by mutableStateOf(false)

    var isSettingsVisible by mutableStateOf(false)
    var isQuickSwitcherVisible by mutableStateOf(false)
    var isSearchVisible by mutableStateOf(false)
    var isServerMenuVisible by mutableStateOf(false)
    var isServerSettingsVisible by mutableStateOf(false)
    var isChannelsAndRolesVisible by mutableStateOf(false)
    var isMediaPickerVisible by mutableStateOf(false)
    var isEmojiPickerVisible by mutableStateOf(false)
    var isPinsVisible by mutableStateOf(false)
    var isThreadPanelVisible by mutableStateOf(false)
    var isProfilePanelVisible by mutableStateOf(true)

    var channelSettingsChannel by mutableStateOf<Channel?>(null)

    fun openChannelSettings(channel: Channel) {
        channelSettingsChannel = channel
    }

    fun closeChannelSettings() {
        channelSettingsChannel = null
    }

    fun openDm(userId: String) {
        val existing = guildStore.privateChannels.value.find { channel ->
            channel.recipients?.any { it.id == userId } == true ||
                channel.recipient_ids?.contains(userId) == true
        }
        if (existing != null) {
            selectedGuild = null
            Settings.shared.clearLastGuild()
            selectChannel(existing, explicitlySelected = true)
            return
        }
        scope.launch {
            val dm = channelApi.openDm(userId)
            if (dm != null) {
                guildStore.handleChannelCreateOrUpdate(dm)
                selectedGuild = null
                Settings.shared.clearLastGuild()
                selectChannel(dm, explicitlySelected = true)
            }
        }
    }

    fun closeDm(channelId: String) {
        scope.launch {
            val closed = channelApi.deleteChannel(channelId)
            if (closed) {
                guildStore.handleChannelDelete(
                    guildStore.privateChannels.value.find { it.id == channelId }?.copy(type = 1)
                        ?: Channel(id = channelId, type = 1)
                )
                if (selectedChannel?.id == channelId) selectHome()
            }
        }
    }

    var isAttachmentViewerVisible by mutableStateOf(false)
    var attachmentViewerItems by mutableStateOf<List<me.lampu.lampcord.shared.model.DiscordMedia>>(emptyList())
    var attachmentViewerIndex by mutableStateOf(0)

    fun openAttachmentViewer(items: List<me.lampu.lampcord.shared.model.DiscordMedia>, index: Int = 0) {
        attachmentViewerItems = items
        attachmentViewerIndex = index
        isAttachmentViewerVisible = true
    }

    fun closeAttachmentViewer() {
        isAttachmentViewerVisible = false
    }
    
    var forwardingMessage by mutableStateOf<me.lampu.lampcord.shared.model.Message?>(null)
    
    var isForumLoading by mutableStateOf(false)

    fun startForwarding(message: me.lampu.lampcord.shared.model.Message) {
        forwardingMessage = message
        isQuickSwitcherVisible = true
    }

    private var guildLoadingJob: Job? = null
    private var channelLoadingJob: Job? = null
    var lastRequestedKey: String? = null

    fun selectHome() {
        guildLoadingJob?.cancel()
        selectedGuild = null
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        lastRequestedKey = null
        Settings.shared.clearLastGuild()
        
        scope.launch {
            // Wait for private channels to be populated if they are empty
            val channels = if (guildStore.privateChannels.value.isEmpty()) {
                guildStore.privateChannels.first { it.isNotEmpty() }
            } else {
                guildStore.privateChannels.value
            }

            val lastDmId = Settings.shared.getLastChannel("home")
            if (lastDmId == "friends") {
                isFriendsSelected = true
                isMentionsSelected = false
                selectedChannel = null
            } else if (lastDmId == "mentions") {
                isFriendsSelected = false
                isMentionsSelected = true
                selectedChannel = null
            } else {
                val dmToSelect = if (lastDmId != null) channels.find { it.id == lastDmId } else channels.firstOrNull()
                if (dmToSelect != null) {
                    selectChannel(dmToSelect)
                } else {
                    isFriendsSelected = true
                    selectedChannel = null
                }
            }
        }
    }

    fun selectFriends() {
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
        isFriendsSelected = true
        isMentionsSelected = false
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        Settings.shared.clearLastGuild()
        Settings.shared.setLastChannel("home", "friends")
    }

    fun selectMentions() {
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
        isFriendsSelected = false
        isMentionsSelected = true
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        Settings.shared.clearLastGuild()
        Settings.shared.setLastChannel("home", "mentions")
    }

    fun restoreLastState(subscribeCallback: (String) -> Unit) {
        val lastGuildId = Settings.shared.getLastGuild()
        val guild = lastGuildId?.let { id -> guildStore.guilds.value.find { it.id == id } }
        if (guild != null) {
            selectGuild(guild, subscribeCallback = subscribeCallback)
        } else {
            selectHome()
        }
    }

    fun selectGuild(guild: Guild, targetChannelId: String? = null, subscribeCallback: (String) -> Unit) {
        if (selectedGuild?.id == guild.id && !isChannelsAndRolesVisible) return
        
        guildLoadingJob?.cancel()
        
        // Optimistic state update
        selectedGuild = guild
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        lastRequestedKey = null
        Settings.shared.setLastGuild(guild.id)

        // Optimistic channel resolution from EntityStore
        val lastChannelId = targetChannelId ?: Settings.shared.getLastChannel(guild.id)
        val allChannels = guildStore.allGuildChannels.value.values
        val optimisticChannel = if (lastChannelId != null) {
            allChannels.find { it.guild_id == guild.id && it.id == lastChannelId }
        } else {
            allChannels.filter { it.guild_id == guild.id && it.type in listOf(0, 2, 5, 13, 15) }
                .minByOrNull { it.position ?: 0 }
        }

        if (optimisticChannel != null) {
            selectChannel(optimisticChannel, explicitlySelected = targetChannelId != null)
        } else {
            isFriendsSelected = false
            selectedChannel = null
            selectedThread = null
            memberListStore.memberListItems.clear()
            memberListStore.onlineCount = null
            memberListStore.memberCount = null
        }

        guildLoadingJob = scope.launch {
            subscribeCallback(guild.id)
            userStore.currentUser.value?.id?.let { userId ->
                val member = guildApi.getGuildMember(guild.id, userId)
                if (member != null) userStore.cacheMember(guild.id, userId, member)
            }
            val guildChannels = channelApi.getGuildChannels(guild.id)
            if (guildChannels.isNotEmpty()) {
                val filtered = guildChannels.filter { it.type in listOf(0, 1, 2, 3, 4, 5, 13, 15, 16) }.sortedBy { it.position }
                filtered.forEach { guildStore.handleChannelCreateOrUpdate(it.copy(guild_id = guild.id)) }
            }
            
            val finalChannelToSelect = if (targetChannelId != null) {
                guildStore.allGuildChannels.value.values.find { it.guild_id == guild.id && it.id == targetChannelId }
                    ?: guildStore.allGuildChannels.value.values.find { it.guild_id == guild.id && it.type in listOf(0, 2, 5, 13, 15) }
            } else {
                val lastId = Settings.shared.getLastChannel(guild.id)
                guildStore.allGuildChannels.value.values.find { it.guild_id == guild.id && it.id == lastId }
                    ?: guildStore.allGuildChannels.value.values.find { it.guild_id == guild.id && it.type in listOf(0, 2, 5, 13, 15) }
            }
            
            if (finalChannelToSelect != null && finalChannelToSelect.id != selectedChannel?.id) {
                selectChannel(finalChannelToSelect, explicitlySelected = targetChannelId != null)
            }
            
            // Onboarding
            selectedGuildOnboarding = null
            if (guild.features?.contains("ONBOARDING") == true) {
                selectedGuildOnboarding = channelApi.getGuildOnboarding(guild.id)
            }
            
            // Fetch commands
            try {
                val index = guildApi.getCommandIndex(guild.id)
                commandStore.clear()
                index?.let {
                    commandStore.setCommands(it.application_commands, it.applications)
                }
            } catch (e: Exception) { }
        }
    }

    fun selectChannel(channel: Channel, explicitlySelected: Boolean = false) {
        val sameChannel = selectedChannel?.id == channel.id
        val hasMessages = messageStore.hasMessages(channel.id)

        if (sameChannel && selectedThread == null && hasMessages) return
        
        isFriendsSelected = false
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        channelLoadingJob?.cancel()
        selectedChannel = channel
        selectedThread = null
        notifier?.dismissChannelNotifications(channel.id)
        if (explicitlySelected) triggerFocusChat()

        // Pre-size the member list store based on expected list ID
        val guild = selectedGuild
        if (guild != null) {
            val expectedId = channel.member_list_id ?: channel.memberListId(guild)
            memberListStore.setExpectedId(guild.id, expectedId, guild.member_count ?: 0)
        }

        if (!sameChannel) {
            messageStore.loadLoggedMessages(channel.id)
        }
        lastRequestedKey = null
        Settings.shared.setLastChannel(selectedGuild?.id ?: "home", channel.id)
        finderStore.addRecent(channel.id)

        channelLoadingJob = scope.launch {
            if (channel.type == 1 || channel.type == 3) {
                val userId = channel.recipients?.firstOrNull()?.id ?: channel.recipient_ids?.firstOrNull()
                if (userId != null) {
                    profileStore.sidebarProfile = null
                    profileStore.isSidebarProfileLoading = true
                    profileStore.sidebarProfile = userApi.getUserProfile(userId)
                    profileStore.isSidebarProfileLoading = false
                }
            } else {
                profileStore.sidebarProfile = null
                profileStore.isSidebarProfileLoading = false
            }

            if (channel.type == 15) {
                isForumLoading = true
                try {
                    val searchThreads = channelApi.searchThreads(channel.id)
                    searchThreads?.threads?.forEach { 
                        guildStore.handleChannelCreateOrUpdate(it.copy(guild_id = channel.guild_id))
                    }
                    
                    val activeThreads = channelApi.getActiveThreads(channel.id)
                    activeThreads?.threads?.forEach { 
                        guildStore.handleChannelCreateOrUpdate(it.copy(guild_id = channel.guild_id))
                    }
                    
                    val archivedThreads = channelApi.getArchivedPublicThreads(channel.id, 50)
                    archivedThreads?.threads?.forEach {
                        guildStore.handleChannelCreateOrUpdate(it.copy(guild_id = channel.guild_id))
                    }
                } catch (e: Exception) {
                    e.printStackTrace()
                } finally {
                    isForumLoading = false
                }
            } else {
                val channelMessages = messageApi.getChannelMessages(channel.id)
                channelMessages.forEach { msg -> 
                    msg.author?.let { author ->
                        msg.member?.let { m -> userStore.cacheMember(channel.guild_id ?: selectedGuild?.id ?: "", author.id, m) }
                    }
                }
                messageStore.addMessages(channel.id, channelMessages)
                channelMessages.firstOrNull()?.let { readStateStore.ackMessage(channel.id, it.id) }
            }
        }
    }

    fun selectThread(channel: Channel, explicitlySelected: Boolean = false) {
        val sameThread = selectedThread?.id == channel.id
        val hasMessages = messageStore.hasMessages(channel.id)

        if (sameThread && hasMessages) return

        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        selectedThread = channel
        if (explicitlySelected) triggerFocusChat()
        messageStore.loadLoggedMessages(channel.id)
        scope.launch {
            val channelMessages = messageApi.getChannelMessages(channel.id)
            channelMessages.forEach { msg -> 
                msg.author?.let { author ->
                    msg.member?.let { m -> userStore.cacheMember(channel.guild_id ?: selectedGuild?.id ?: "", author.id, m) }
                }
            }
            messageStore.addMessages(channel.id, channelMessages)
            channelMessages.firstOrNull()?.let { readStateStore.ackMessage(channel.id, it.id) }
        }
    }

    fun selectChannelById(channelId: String, explicitlySelected: Boolean = false) {
        val existing = guildStore.allGuildChannels.value[channelId] ?: guildStore.privateChannels.value.find { it.id == channelId }
        if (existing != null) {
            if (existing.type in listOf(10, 11, 12)) {
                selectThread(existing, explicitlySelected = explicitlySelected)
            } else {
                selectChannel(existing, explicitlySelected = explicitlySelected)
            }
        } else {
            scope.launch {
                val fetched = channelApi.getChannel(channelId)
                if (fetched != null) {
                    guildStore.handleChannelCreateOrUpdate(fetched)
                    if (fetched.type in listOf(10, 11, 12)) {
                        selectThread(fetched, explicitlySelected = explicitlySelected)
                    } else {
                        selectChannel(fetched, explicitlySelected = explicitlySelected)
                    }
                }
            }
        }
    }

    fun clear() {
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
        selectedGuildOnboarding = null
        isFriendsSelected = false
        isMentionsSelected = false
        isConnected = false
        isConnecting = false
        isSettingsVisible = false
        isQuickSwitcherVisible = false
        isSearchVisible = false
        isServerSettingsVisible = false
        isChannelsAndRolesVisible = false
        isMediaPickerVisible = false
        isPinsVisible = false
        isForumLoading = false
        guildLoadingJob?.cancel()
        channelLoadingJob?.cancel()
        lastRequestedKey = null
    }
}
