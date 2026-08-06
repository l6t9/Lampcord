package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.settings.Settings
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class SavedAccount(
    val token: String,
    val user: User
)

class TokenStore(private val json: Json) {
    private var accounts = mutableStateListOf<SavedAccount>()

    init {
        load()
    }

    private fun load() {
        try {
            val jsonStr = Settings.shared.savedAccountsJson
            if (jsonStr.isNotBlank() && jsonStr != "[]") {
                val loaded = json.decodeFromString<List<SavedAccount>>(jsonStr)
                accounts.clear()
                accounts.addAll(loaded)
            }
        } catch (_: Exception) {
            // Silence error
        }
    }

    private fun save() {
        try {
            val list = accounts.toList()
            val jsonStr = json.encodeToString(list)
            Settings.shared.savedAccountsJson = jsonStr
        } catch (_: Exception) {
            // Silence error
        }
    }

    fun getAccounts(): List<SavedAccount> = accounts

    fun addAccount(token: String, user: User) {
        val existingIndex = accounts.indexOfFirst { it.user.id == user.id }
        if (existingIndex != -1) {
            accounts[existingIndex] = SavedAccount(token, user)
        } else {
            accounts.add(SavedAccount(token, user))
        }
        save()
    }

    fun removeAccount(token: String) {
        accounts.removeAll { it.token == token }
        save()
    }

    fun switchAccount(token: String) {
        Settings.shared.discordToken = token
    }
}
