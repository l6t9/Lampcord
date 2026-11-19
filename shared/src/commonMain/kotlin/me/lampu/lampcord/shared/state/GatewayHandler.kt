package me.lampu.lampcord.shared.state

import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.utils.Logging

class GatewayHandler(
    private val json: Json,
    private val dispatcher: GatewayEventDispatcher,
    private val scope: CoroutineScope,
    private val userStore: UserStore,
    private val settingsStore: SettingsStore,
    private val userGuildSettingsStore: UserGuildSettingsStore,
    private val guildStore: GuildStore,
    private val messageStore: MessageStore,
    private val readStateStore: ReadStateStore,
    private val experimentStore: ExperimentStore,
    private val relationshipStore: RelationshipStore,
    private val presenceStore: PresenceStore,
    private val navigationStore: NavigationStore,
    private val gatewayManager: GatewayManager,
    private val tokenStore: TokenStore,
    private val memberListStore: MemberListStore,
    private val voiceStore: VoiceStore
) {
    fun handleGatewayEvent(payload: GatewayPayload) {
        when (payload.t) {
            "READY" -> handleReady(payload)
            "RESUMED" -> {
                navigationStore.isConnected = true
                navigationStore.isConnecting = false
                messageStore.handleConnected()
                // destroy() no longer wipes subscriptions on reconnect, but replay
                // them so Discord re-delivers events for all tracked channels/threads.
                gatewayManager.replaySubscriptions()
                memberListStore.resubscribe()
            }
            else -> dispatcher.dispatch(payload)
        }
    }

    private fun handleReady(payload: GatewayPayload) {
        payload.d?.let { data ->
            val user = try {
                data.jsonObject["user"]?.let { json.decodeFromJsonElement<User>(it) }
            } catch (e: Exception) { null }

            user?.let { u ->
                userStore.setCurrentUser(u, data.jsonObject["user"]?.jsonObject)
                val token = Settings.shared.discordToken
                if (token.isNotBlank()) {
                    tokenStore.addAccount(token, u)
                }
            }

            // Mark the session usable before decoding optional READY fields. If any
            // optional field fails to decode, the client must not stay stuck in the
            // "connecting" state with an undrained outgoing message queue.
            navigationStore.isConnected = true
            navigationStore.isConnecting = false
            messageStore.handleConnected()

            try {
                val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                
                if (ready.user_settings != null && ready.user_settings is JsonObject) {
                    val settings = json.decodeFromJsonElement<UserSettings>(ready.user_settings)
                    settingsStore.userSettings = settings
                    // READY is an ingestion path too; do not bypass appearance sync.
                    settingsStore.applyDiscordAppearance(settings.theme)
                }
                
                userGuildSettingsStore.handleReady(ready)
                
                val settings = settingsStore.userSettings
                val guildOrder = settings?.guild_folders?.flatMap { folder ->
                    folder.guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull }
                }?.takeIf { it.isNotEmpty() }
                    ?: settings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() }
                    ?: emptyList()
                
                guildStore.setGuilds(ready.guilds, guildOrder)
                guildStore.setPrivateChannels(ready.private_channels)

                ready.merged_members?.forEachIndexed { index, members ->
                    val guild = ready.guilds.getOrNull(index) ?: return@forEachIndexed
                    val membersJson = data.jsonObject["merged_members"]?.jsonArray?.getOrNull(index)?.jsonArray
                    val pairs = ArrayList<Pair<String, Member>>(members.size)
                    val raws = ArrayList<JsonObject?>(members.size)
                    members.forEachIndexed { memberIndex, member ->
                        val userId = member.userId() ?: return@forEachIndexed
                        pairs.add(userId to member)
                        raws.add(membersJson?.getOrNull(memberIndex)?.jsonObject)
                    }
                    userStore.cacheMembers(guild.id, pairs, raws)
                }

                ready.users?.let { users ->
                    val usersJson = data.jsonObject["users"]?.jsonArray
                    userStore.handleUserUpdates(
                        users.mapIndexed { index, user -> user to usersJson?.getOrNull(index)?.jsonObject }
                    )
                }

                presenceStore.handleReady(ready)
                ready.sessions?.let { sessions ->
                    user?.id?.let { presenceStore.handleSessions(it, sessions) }
                }

                readStateStore.handleReady(ready)
                experimentStore.handleReady(ready.experiments)
                ready.relationships?.let { relationshipStore.handleReady(it) }

                voiceStore.handleReady(data.jsonObject)

                if (navigationStore.selectedGuild == null && navigationStore.selectedChannel == null && !navigationStore.isFriendsSelected) {
                    navigationStore.restoreLastState { gatewayManager.sendSubscription(it) }
                }

                memberListStore.resubscribe()
            } catch (e: Exception) {
                Logging.e("GatewayHandler", "Error processing READY payload", e)
            }
        }
    }
}
