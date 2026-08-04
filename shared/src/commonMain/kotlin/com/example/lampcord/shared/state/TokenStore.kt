package com.example.lampcord.shared.state

import com.example.lampcord.shared.model.User
import com.example.lampcord.shared.settings.Settings
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class SavedAccount(
    val token: String,
    val user: User
)

class TokenStore(private val json: Json) {
    private var accounts: MutableList<SavedAccount> = mutableListOf()

    init {
        load()
    }

    private fun load() {
        try {
            val jsonStr = Settings.shared.savedAccountsJson
            accounts = json.decodeFromString<List<SavedAccount>>(jsonStr).toMutableList()
        } catch (e: Exception) {
            accounts = mutableListOf()
        }
    }

    private fun save() {
        Settings.shared.savedAccountsJson = json.encodeToString(accounts)
    }

    fun getAccounts(): List<SavedAccount> = accounts

    fun addAccount(token: String, user: User) {
        if (accounts.none { it.token == token }) {
            accounts.add(SavedAccount(token, user))
            save()
        }
    }

    fun removeAccount(token: String) {
        accounts.removeAll { it.token == token }
        save()
    }

    fun switchAccount(token: String) {
        Settings.shared.discordToken = token
    }
}
