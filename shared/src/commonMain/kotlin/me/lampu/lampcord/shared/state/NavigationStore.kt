package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Onboarding
import me.lampu.lampcord.shared.model.memberListId
import me.lampu.lampcord.shared.settings.Settings

class NavigationStore(
    private val discordClient: DiscordClient,
    private val guildStore: GuildStore,
    private val memberListStore: MemberListStore,
    private val messageStore: MessageStore,
    private val userStore: UserStore,
    private val readStateStore: ReadStateStore,
    private val profileStore: ProfileStore,
    private val commandStore: CommandStore,
    private val selectionStore: SelectionStore,
    private val finderStore: FinderStore,
    private val scope: CoroutineScope
) {
    var selectedGuild by selectionStore::selectedGuild
    var selectedChannel by selectionStore::selectedChannel
    var selectedThread by selectionStore::selectedThread

    var selectedGuildOnboarding by mutableStateOf<Onboarding?>(null)
    var isFriendsSelected by mutableStateOf(false)

    var isConnected by mutableStateOf(false)
    var isConnecting by mutableStateOf(false)

    var isSettingsVisible by mutableStateOf(false)
    var isQuickSwitcherVisible by mutableStateOf(false)
    var isSearchVisible by mutableStateOf(false)
    var isServerMenuVisible by mutableStateOf(false)
    var isServerSettingsVisible by mutableStateOf(false)
    var isChannelsAndRolesVisible by mutableStateOf(false)
    var isMediaPickerVisible by mutableStateOf(false)
    var isPinsVisible by mutableStateOf(false)

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
        selectedGuild = null
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        memberListStore.clear()
        lastRequestedKey = null
        
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
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        Settings.shared.setLastChannel("home", "friends")
    }

    fun selectGuild(guild: Guild, subscribeCallback: (String) -> Unit) {
        if (selectedGuild?.id == guild.id && !isChannelsAndRolesVisible) return
        guildLoadingJob?.cancel()
        selectedGuild = guild
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        memberListStore.clear()
        lastRequestedKey = null
        guildLoadingJob = scope.launch {
            subscribeCallback(guild.id)
            val guildChannels = discordClient.getGuildChannels(guild.id)
            if (guildChannels.isNotEmpty()) {
                val filtered = guildChannels.filter { it.type in listOf(0, 1, 2, 3, 4, 5, 13, 15, 16) }.sortedBy { it.position }
                filtered.forEach { guildStore.handleChannelCreateOrUpdate(it.copy(guild_id = guild.id)) }
            }
            val lastChannelId = Settings.shared.getLastChannel(guild.id)
            val channelToSelect = guildStore.allGuildChannels.value.values.find { it.guild_id == guild.id && it.id == lastChannelId } 
                ?: guildStore.allGuildChannels.value.values.find { it.guild_id == guild.id && it.type in listOf(0, 2, 5, 13, 15) }
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

    fun selectChannel(channel: Channel) {
        if (selectedChannel?.id == channel.id) return
        isFriendsSelected = false
        isChannelsAndRolesVisible = false
        isServerSettingsVisible = false
        channelLoadingJob?.cancel()
        selectedChannel = channel
        selectedThread = null

        // Pre-size the member list store based on expected list ID
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
        finderStore.addRecent(channel.id)

        channelLoadingJob = scope.launch {
            if (channel.type == 1 || channel.type == 3) {
                val userId = channel.recipients?.firstOrNull()?.id ?: channel.recipient_ids?.firstOrNull()
                if (userId != null) {
                    profileStore.sidebarProfile = null
                    profileStore.isSidebarProfileLoading = true
                    profileStore.sidebarProfile = discordClient.getUserProfile(userId)
                    profileStore.isSidebarProfileLoading = false
                }
            } else {
                profileStore.sidebarProfile = null
                profileStore.isSidebarProfileLoading = false
            }

            if (channel.type == 15) {
                isForumLoading = true
                try {
                    // Logic for forum threads if needed
                } finally {
                    isForumLoading = false
                }
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

    fun clear() {
        selectedGuild = null
        selectedChannel = null
        selectedThread = null
        selectedGuildOnboarding = null
        isFriendsSelected = false
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
