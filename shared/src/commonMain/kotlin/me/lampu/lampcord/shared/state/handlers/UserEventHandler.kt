package me.lampu.lampcord.shared.state.handlers

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.decodeFromJsonElement
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.model.UserSettings
import me.lampu.lampcord.shared.model.UserNoteUpdate
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.state.GatewayEventHandler
import me.lampu.lampcord.shared.state.SettingsStore

class UserEventHandler(
    private val json: Json,
    private val userStore: UserStore,
    private val settingsStore: SettingsStore
) : GatewayEventHandler {
    override val supportedEvents = setOf("USER_UPDATE", "USER_SETTINGS_UPDATE", "USER_NOTE_UPDATE")

    override fun handleEvent(type: String, data: JsonElement?) {
        if (data == null) return
        when (type) {
            "USER_UPDATE" -> handleUserUpdate(data)
            "USER_SETTINGS_UPDATE" -> handleUserSettingsUpdate(data)
            "USER_NOTE_UPDATE" -> handleUserNoteUpdate(data)
        }
    }

    private fun handleUserUpdate(data: JsonElement) {
        try {
            val user = json.decodeFromJsonElement<User>(data)
            userStore.handleUserUpdate(user)
        } catch (e: Exception) { }
    }

    private fun handleUserSettingsUpdate(data: JsonElement) {
        try {
            val settings = json.decodeFromJsonElement<UserSettings>(data)
            settingsStore.handleUserSettingsUpdate(settings)
        } catch (e: Exception) { }
    }

    private fun handleUserNoteUpdate(data: JsonElement) {
        try {
            val update = json.decodeFromJsonElement<UserNoteUpdate>(data)
            // handle note update if we ever store notes
        } catch (e: Exception) { }
    }
}
