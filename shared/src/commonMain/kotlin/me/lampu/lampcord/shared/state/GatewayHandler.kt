package me.lampu.lampcord.shared.state

import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.settings.Settings

class GatewayHandler(
    private val json: Json,
    private val discordClient: DiscordClient,
    private val scope: CoroutineScope,
    private val userStore: UserStore,
    private val tokenStore: TokenStore,
    private val settingsStore: SettingsStore,
    private val userGuildSettingsStore: UserGuildSettingsStore,
    private val guildStore: GuildStore,
    private val readStateStore: ReadStateStore,
    private val experimentStore: ExperimentStore,
    private val relationshipStore: RelationshipStore,
    private val presenceStore: PresenceStore,
    private val messageStore: MessageStore,
    private val memberListStore: MemberListStore,
    private val typingStore: TypingStore,
    private val voiceStore: VoiceStore,
    private val navigationStore: NavigationStore,
    private val finderStore: FinderStore,
    private val gatewayManager: GatewayManager
) {
    fun handleGatewayEvent(payload: GatewayPayload) {
        when (payload.t) {
            "READY" -> handleReady(payload)
            "GUILD_CREATE" -> handleGuildCreate(payload)
            "GUILD_UPDATE" -> handleGuildUpdate(payload)
            "GUILD_DELETE" -> handleGuildDelete(payload)
            "CHANNEL_UPDATE" -> handleChannelUpdate(payload)
            "CHANNEL_DELETE" -> handleChannelDelete(payload)
            "GUILD_ROLE_UPDATE" -> handleRoleUpdate(payload)
            "GUILD_ROLE_DELETE" -> handleRoleDelete(payload)
            "MESSAGE_CREATE" -> handleMessageCreate(payload)
            "MESSAGE_UPDATE" -> handleMessageUpdate(payload)
            "MESSAGE_DELETE" -> handleMessageDelete(payload)
            "MESSAGE_ACK" -> handleMessageAck(payload)
            "GUILD_MEMBER_LIST_UPDATE" -> handleMemberListUpdate(payload)
            "THREAD_LIST_SYNC" -> handleThreadListSync(payload)
            "THREAD_UPDATE" -> handleThreadUpdate(payload)
            "THREAD_DELETE" -> handleThreadDelete(payload)
            "RELATIONSHIP_ADD" -> handleRelationshipAdd(payload)
            "RELATIONSHIP_REMOVE" -> handleRelationshipRemove(payload)
            "USER_SETTINGS_UPDATE" -> handleUserSettingsUpdate(payload)
            "PRESENCE_UPDATE" -> handlePresenceUpdate(payload)
            "USER_UPDATE" -> handleUserUpdate(payload)
            "USER_NOTE_UPDATE" -> handleUserNoteUpdate(payload)
            "TYPING_START" -> handleTypingStart(payload)
            "MESSAGE_REACTION_ADD" -> handleReactionAddEvent(payload)
            "MESSAGE_REACTION_REMOVE" -> handleReactionRemoveEvent(payload)
            "VOICE_STATE_UPDATE" -> voiceStore.handleVoiceStateUpdate(payload, userStore.currentUser?.id)
            "VOICE_SERVER_UPDATE" -> voiceStore.handleVoiceServerUpdate(payload, userStore.currentUser?.id)
            "USER_GUILD_SETTINGS_UPDATE" -> handleUserGuildSettingsUpdate(payload)
            "RESUMED" -> {
                navigationStore.isConnected = true
                navigationStore.isConnecting = false
                println("Session resumed successfully")
            }
        }
    }

    private fun handleTypingStart(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val typing = json.decodeFromJsonElement<TypingStart>(data)
                
                // Cache member/user if provided (typical in guilds)
                typing.guild_id?.let { guildId ->
                    typing.member?.let { member ->
                        userStore.cacheMember(guildId, typing.user_id, member)
                    }
                }

                typingStore.handleTypingStart(typing.channel_id, typing.user_id, userStore.currentUser?.id)
            } catch (e: Exception) { }
        }
    }

    private fun handleReactionAddEvent(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val add = json.decodeFromJsonElement<MessageReactionAdd>(data)
                val index = messageStore.messages.indexOfFirst { it.id == add.message_id }
                if (index != -1) {
                    val msg = messageStore.messages[index]
                    val reactions = msg.reactions?.toMutableList() ?: mutableListOf()
                    val emojiIndex = reactions.indexOfFirst { it.emoji.id == add.emoji.id && it.emoji.name == add.emoji.name }
                    val isMe = add.user_id == userStore.currentUser?.id
                    if (emojiIndex != -1) {
                        val r = reactions[emojiIndex]
                        reactions[emojiIndex] = r.copy(count = r.count + 1, me = if (isMe) true else r.me)
                    } else {
                        reactions.add(MessageReaction(emoji = add.emoji, count = 1, me = isMe, me_burst = false, count_details = ReactionCountDetails(0, 1)))
                    }
                    messageStore.messages[index] = msg.copy(reactions = reactions)
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleReactionRemoveEvent(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val remove = json.decodeFromJsonElement<MessageReactionRemove>(data)
                val index = messageStore.messages.indexOfFirst { it.id == remove.message_id }
                if (index != -1) {
                    val msg = messageStore.messages[index]
                    val reactions = msg.reactions?.toMutableList() ?: return
                    val emojiIndex = reactions.indexOfFirst { it.emoji.id == remove.emoji.id && it.emoji.name == remove.emoji.name }
                    if (emojiIndex != -1) {
                        val r = reactions[emojiIndex]
                        val isMe = remove.user_id == userStore.currentUser?.id
                        val newCount = (r.count - 1).coerceAtLeast(0)
                        if (newCount == 0) {
                            reactions.removeAt(emojiIndex)
                        } else {
                            reactions[emojiIndex] = r.copy(count = newCount, me = if (isMe) false else r.me)
                        }
                        messageStore.messages[index] = msg.copy(reactions = reactions)
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleUserGuildSettingsUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val partial = json.decodeFromJsonElement<UserGuildSettings.Partial>(data)
                userGuildSettingsStore.handlePartialUpdate(partial)
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
                userStore.currentUser = u
                userStore.handleUserUpdate(u)
                // tokenStore handling could go here if currentToken was available
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

                navigationStore.isConnected = true
                navigationStore.isConnecting = false

                if (navigationStore.selectedGuild == null && navigationStore.selectedChannel == null) {
                    navigationStore.selectHome()
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
                val partial = json.decodeFromJsonElement<Guild.Partial>(data)
                val guildId = data.jsonObject["id"]?.jsonPrimitive?.content ?: return
                guildStore.guilds.find { it.id == guildId }?.let { existing ->
                    val updated = existing.merge(partial)
                    val index = guildStore.guilds.indexOf(existing)
                    if (index != -1) guildStore.guilds[index] = updated
                    if (navigationStore.selectedGuild?.id == guildId) navigationStore.selectedGuild = updated
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleGuildDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            val guildId = data.jsonObject["id"]?.jsonPrimitive?.content ?: return
            guildStore.handleGuildDelete(guildId)
            if (navigationStore.selectedGuild?.id == guildId) {
                navigationStore.selectHome()
            }
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
                val guildId = data.jsonObject["guild_id"]?.jsonPrimitive?.content ?: return
                val role = data.jsonObject["role"]?.let { json.decodeFromJsonElement<Role>(it) } ?: return
                guildStore.handleRoleCreateOrUpdate(guildId, role)
            } catch (e: Exception) { }
        }
    }

    private fun handleRoleDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val guildId = data.jsonObject["guild_id"]?.jsonPrimitive?.content ?: return
                val roleId = data.jsonObject["role_id"]?.jsonPrimitive?.content ?: return
                guildStore.handleRoleDelete(guildId, roleId)
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageCreate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val message = json.decodeFromJsonElement<Message>(data)
                
                // Update user/member caches
                message.guild_id?.let { guildId ->
                    message.member?.let { member ->
                        message.author?.let { author ->
                            userStore.cacheMember(guildId, author.id, member)
                        }
                    }
                }
                message.author?.let { userStore.handleUserUpdate(it) }
                finderStore.addRecent(message.channel_id)

                // Update DM order if it's a DM
                if (message.guild_id == null) {
                    val dm = guildStore.privateChannels.find { it.id == message.channel_id }
                    if (dm != null) {
                        guildStore.privateChannels.remove(dm)
                        guildStore.privateChannels.add(0, dm.copy(last_message_id = JsonPrimitive(message.id)))
                    }
                }

                if (navigationStore.selectedChannel?.id == message.channel_id || navigationStore.selectedThread?.id == message.channel_id) {
                    messageStore.handleMessageCreate(message)
                    scope.launch {
                        readStateStore.ackMessage(message.channel_id, message.id)
                    }
                } else {
                    val currentMember = navigationStore.selectedGuild?.let { userStore.getMember(it.id, userStore.currentUser?.id ?: "") }
                    if (messageStore.isMessageMentioningMe(message, userStore.currentUser, currentMember)) {
                        val state = readStateStore.readStates[message.channel_id]
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
                memberListStore.handleMemberListUpdate(update)

                // 126.21 Parity: Update presence store with members that have presences
                update.ops.forEach { op ->
                    op.items?.forEach { item ->
                        item.member?.presence?.let { presence ->
                            presenceStore.handlePresenceUpdate(presence)
                        }
                    }
                    op.item?.member?.presence?.let { presence ->
                        presenceStore.handlePresenceUpdate(presence)
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleThreadListSync(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val sync = json.decodeFromJsonElement<ThreadListSync>(data)
                val forum = navigationStore.selectedChannel ?: return
                if (forum.type == 15 && sync.guild_id == navigationStore.selectedGuild?.id) {
                    if (sync.channel_ids == null || forum.id in sync.channel_ids) {
                        sync.threads.filter { it.parent_id == forum.id }.forEach { guildStore.upsertForumThread(it) }
                    }
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleThreadUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val thread = json.decodeFromJsonElement<Channel>(data)
                guildStore.upsertForumThread(thread)
            } catch (e: Exception) { }
        }
    }

    private fun handleThreadDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val thread = json.decodeFromJsonElement<Channel>(data)
                guildStore.handleChannelDelete(thread)
                if (navigationStore.selectedThread?.id == thread.id) {
                    navigationStore.selectedThread = null
                }
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val message = json.decodeFromJsonElement<Message>(data)
                messageStore.handleMessageUpdate(message, data.jsonObject)
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageDelete(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return
                messageStore.handleMessageDelete(id)
            } catch (e: Exception) { }
        }
    }

    private fun handleRelationshipAdd(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val rel = json.decodeFromJsonElement<Relationship>(data)
                relationshipStore.handleRelationshipAdd(rel)
            } catch (e: Exception) { }
        }
    }

    private fun handleRelationshipRemove(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val id = data.jsonObject["id"]?.jsonPrimitive?.content ?: return
                relationshipStore.handleRelationshipRemove(id)
            } catch (e: Exception) { }
        }
    }

    private fun handleUserSettingsUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val settings = json.decodeFromJsonElement<UserSettings>(data)
                settingsStore.handleUserSettingsUpdate(settings)
            } catch (e: Exception) { }
        }
    }

    private fun handlePresenceUpdate(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val presence = json.decodeFromJsonElement<PresenceUpdate>(data)
                presenceStore.handlePresenceUpdate(presence)
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
                // handle note update if we ever store notes
            } catch (e: Exception) { }
        }
    }

    private fun handleMessageAck(payload: GatewayPayload) {
        payload.d?.let { data ->
            try {
                val ack = json.decodeFromJsonElement<MessageAcknowledge>(data)
                readStateStore.handleMessageAck(ack)
            } catch (e: Exception) { }
        }
    }
}
