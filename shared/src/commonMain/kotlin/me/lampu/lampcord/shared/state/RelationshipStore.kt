package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Relationship

class RelationshipStore(
    private val discordClient: DiscordClient,
    private val scope: CoroutineScope
) {
    val relationships = mutableStateListOf<Relationship>()

    fun fetchRelationships() {
        scope.launch {
            val friends = discordClient.getRelationships()
            relationships.clear()
            relationships.addAll(friends)
        }
    }

    fun handleRelationshipAdd(rel: Relationship) {
        relationships.removeAll { it.id == rel.id }
        relationships.add(rel)
    }

    fun handleRelationshipRemove(id: String) {
        relationships.removeAll { it.id == id }
    }

    fun addFriend(userId: String) {
        scope.launch {
            discordClient.addRelationship(userId, 1)
        }
    }

    fun removeFriend(userId: String) {
        scope.launch {
            discordClient.deleteRelationship(userId)
        }
    }

    fun blockUser(userId: String) {
        scope.launch {
            discordClient.addRelationship(userId, 2)
        }
    }

    fun unblockUser(userId: String) {
        scope.launch {
            discordClient.deleteRelationship(userId)
        }
    }

    fun clear() {
        relationships.clear()
    }
}
