package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.contentOrNull
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.model.UserNoteUpdate
import me.lampu.lampcord.shared.state.*

class UserEventHandler(
    private val json: Json,
    private val userStore: UserStore,
    private val settingsStore: SettingsStore,
    private val guildStore: GuildStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("USER_UPDATE", "USER_SETTINGS_UPDATE", "USER_NOTE_UPDATE", "GUILD_MEMBER_UPDATE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "USER_UPDATE" -> handleUserUpdate(data)
            "USER_SETTINGS_UPDATE" -> handleUserSettingsUpdate(data)
            "USER_NOTE_UPDATE" -> handleUserNoteUpdate(data)
            "GUILD_MEMBER_UPDATE" -> handleGuildMemberUpdate(data)
        }
    }

    private fun handleUserUpdate(data: JsonElement) {
        try {
            val user = json.decodeFromJsonElement<User>(data)
            userStore.handleUserUpdate(user, data.jsonObject)
        } catch (e: Exception) { }
    }

    private fun handleUserSettingsUpdate(data: JsonElement) {
        try {
            val settings = json.decodeFromJsonElement<UserSettings>(data)
            settingsStore.handleUserSettingsUpdate(settings)
            
            val guildOrder = settings.guild_folders.flatMap { folder ->
                folder.guild_ids.mapNotNull { it.jsonPrimitive.contentOrNull }
            }.takeIf { it.isNotEmpty() }
                ?: settings.guild_positions.mapNotNull { it.jsonPrimitive.contentOrNull }
            
            guildStore.reorderGuilds(guildOrder)
        } catch (e: Exception) { }
    }

    private fun handleUserNoteUpdate(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<UserNoteUpdate>(data)
        } catch (e: Exception) { }
    }

    private fun handleGuildMemberUpdate(data: JsonElement) {
        try {
            val member = json.decodeFromJsonElement<Member>(data)
            val guildId = data.jsonObject["guild_id"]?.jsonPrimitive?.content ?: return
            val userId = member.userId() ?: return
            userStore.cacheMember(guildId, userId, member, data.jsonObject)
        } catch (e: Exception) { }
    }
}
