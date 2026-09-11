package me.lampu.lampcord.shared.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.Relationship

class RelationshipStore(
    private val userApi: UserApi,
    private val userStore: UserStore,
    private val scope: CoroutineScope
) {
    private val _relationships = MutableStateFlow<List<Relationship>>(emptyList())
    val relationships: StateFlow<List<Relationship>> = _relationships.asStateFlow()

    val relationshipTypes: StateFlow<Map<String, Int>> = _relationships
        .map { list ->
            HashMap<String, Int>(list.size * 2).also { map ->
                for (rel in list) {
                    val id = rel.id ?: rel.user?.id ?: rel.user_id ?: continue
                    val type = rel.type ?: continue
                    map[id] = type
                }
            }
        }
        .stateIn(scope, SharingStarted.Eagerly, emptyMap())

    fun handleReady(rels: List<Relationship>) {
        userStore.handleUserUpdates(rels.mapNotNull { rel -> rel.user?.let { it to null } })
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
            val friends = userApi.getRelationships()
            _relationships.value = friends.map { hydrate(it) }.distinctBy { it.id ?: it.user?.id ?: it.user_id }
        }
    }

    fun handleRelationshipAdd(rel: Relationship) {
        rel.user?.let { userStore.handleUserUpdate(it) }
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
            userApi.addRelationship(userId, 1)
        }
    }

    fun sendFriendRequest(username: String, discriminator: String?, onResult: (Boolean) -> Unit = {}) {
        scope.launch {
            val success = userApi.addRelationshipByUsername(username, discriminator)
            if (success) fetchRelationships()
            onResult(success)
        }
    }

    fun removeFriend(userId: String) {
        scope.launch {
            userApi.deleteRelationship(userId)
        }
    }

    fun updateNickname(userId: String, nickname: String?) {
        scope.launch {
            if (userApi.updateRelationship(userId, nickname)) {
                fetchRelationships()
            }
        }
    }

    fun blockUser(userId: String) {
        scope.launch {
            userApi.addRelationship(userId, 2)
        }
    }

    fun unblockUser(userId: String) {
        scope.launch {
            userApi.deleteRelationship(userId)
        }
    }

    fun clear() {
        _relationships.value = emptyList()
    }
}
