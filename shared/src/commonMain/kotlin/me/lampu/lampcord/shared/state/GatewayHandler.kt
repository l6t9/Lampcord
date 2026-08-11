package me.lampu.lampcord.shared.state

import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.settings.Settings

/**
 * GatewayHandler handles top-level gateway events and orchestrates store updates,
 * matching the event flow in Discord's GatewayHandler and Store architecture.
 */
class GatewayHandler(
    private val json: Json,
    private val dispatcher: GatewayEventDispatcher,
    private val scope: CoroutineScope,
    private val userStore: UserStore,
    private val settingsStore: SettingsStore,
    private val userGuildSettingsStore: UserGuildSettingsStore,
    private val guildStore: GuildStore,
    private val readStateStore: ReadStateStore,
    private val experimentStore: ExperimentStore,
    private val relationshipStore: RelationshipStore,
    private val presenceStore: PresenceStore,
    private val navigationStore: NavigationStore,
    private val discordClient: DiscordClient,
    private val gatewayManager: GatewayManager,
    private val tokenStore: TokenStore
) {
    fun handleGatewayEvent(payload: GatewayPayload) {
        when (payload.t) {
            "READY" -> handleReady(payload)
            "RESUMED" -> {
                navigationStore.isConnected = true
                navigationStore.isConnecting = false
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
                userStore.setCurrentUser(u)
                val token = Settings.shared.discordToken
                if (token.isNotBlank()) {
                    tokenStore.addAccount(token, u)
                }
            }

            try {
                val ready = json.decodeFromJsonElement<ReadyPayload>(data)
                
                if (ready.user_settings != null && ready.user_settings is JsonObject) {
                    val settings = json.decodeFromJsonElement<UserSettings>(ready.user_settings)
                    settingsStore.userSettings = settings
                }
                
                userGuildSettingsStore.handleReady(ready)
                
                val guildOrder = settingsStore.userSettings?.guild_positions?.mapNotNull { it.jsonPrimitive.contentOrNull ?: it.toString() } ?: emptyList()
                
                // 1. Process guilds
                guildStore.setGuilds(ready.guilds, guildOrder)
                
                // 2. Process private channels (DMs)
                guildStore.setPrivateChannels(ready.private_channels)
                
                // 3. Process merged members (StoreMembers)
                ready.merged_members?.forEachIndexed { index, members ->
                    val guild = ready.guilds.getOrNull(index) ?: return@forEachIndexed
                    members.forEach { member ->
                        val userId = member.userId() ?: return@forEach
                        userStore.cacheMember(guild.id, userId, member)
                    }
                }
                
                // 4. Process global users (StoreUsers)
                ready.users?.forEach { userStore.handleUserUpdate(it) }

                presenceStore.handleReady(ready)
                readStateStore.handleReady(ready)
                experimentStore.handleReady(ready.experiments)
                ready.relationships?.let { relationshipStore.handleReady(it) }

                gatewayManager.sendVoiceStateUpdate(
                    guildId = null,
                    channelId = null,
                    selfMute = true,
                    selfDeaf = true
                )
                
                navigationStore.isConnected = true
                navigationStore.isConnecting = false

                // Auto-select last channel/DM on startup
                if (navigationStore.selectedGuild == null && navigationStore.selectedChannel == null && !navigationStore.isFriendsSelected) {
                    navigationStore.selectHome()
                }
            } catch (e: Exception) { }
        }
    }
}
