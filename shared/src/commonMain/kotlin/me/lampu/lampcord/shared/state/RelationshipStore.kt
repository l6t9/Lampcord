package me.lampu.lampcord.shared.state

import androidx.compose.runtime.mutableStateListOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Relationship

class RelationshipStore(
    private val discordClient: DiscordClient,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) {
    val relationships = mutableStateListOf<Relationship>()

    fun handleReady(rels: List<Relationship>) {
        println("RelationshipStore received ${rels.size} relationships")
        relationships.clear()
        relationships.addAll(rels.map { hydrate(it) })
    }

    private fun hydrate(rel: Relationship): Relationship {
        if (rel.user != null) return rel
        val userId = rel.user_id ?: rel.id ?: return rel
        val cachedUser = userStore.getUser(userId)
        return if (cachedUser != null) rel.copy(user = cachedUser) else rel
    }

    fun fetchRelationships() {
        scope.launch {
            val friends = discordClient.getRelationships()
            relationships.clear()
            relationships.addAll(friends.map { hydrate(it) })
        }
    }

    fun handleRelationshipAdd(rel: Relationship) {
        val hydrated = hydrate(rel)
        val id = hydrated.id ?: hydrated.user?.id ?: hydrated.user_id
        relationships.removeAll { (it.id ?: it.user?.id ?: it.user_id) == id }
        relationships.add(hydrated)
    }

    fun handleRelationshipRemove(id: String) {
        relationships.removeAll { (it.id ?: it.user?.id ?: it.user_id) == id }
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
