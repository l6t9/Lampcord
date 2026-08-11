package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.DiscordClient
import me.lampu.lampcord.shared.model.Relationship

class RelationshipStore(
    private val discordClient: DiscordClient,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) {
    private val _relationships = MutableStateFlow<List<Relationship>>(emptyList())
    val relationships: StateFlow<List<Relationship>> = _relationships.asStateFlow()

    fun handleReady(rels: List<Relationship>) {
        println("RelationshipStore received ${rels.size} relationships")
        _relationships.value = rels.map { hydrate(it) }.distinctBy { it.id ?: it.user?.id ?: it.user_id }
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
            _relationships.value = friends.map { hydrate(it) }.distinctBy { it.id ?: it.user?.id ?: it.user_id }
        }
    }

    fun handleRelationshipAdd(rel: Relationship) {
        val hydrated = hydrate(rel)
        val id = hydrated.id ?: hydrated.user?.id ?: hydrated.user_id
        _relationships.update { current ->
            current.filterNot { (it.id ?: it.user?.id ?: it.user_id) == id } + hydrated
        }
    }

    fun handleRelationshipRemove(id: String) {
        _relationships.update { current ->
            current.filterNot { (it.id ?: it.user?.id ?: it.user_id) == id }
        }
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
        _relationships.value = emptyList()
    }
}
