package me.lampu.lampcord.shared.state

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.User
import kotlin.test.Test
import kotlin.test.assertEquals

class FinderMatchingTest {

    private val users = listOf(
        User(id = "1", username = "alice", global_name = "Alice Smith"),
        User(id = "2", username = "bob-offline"),
        User(id = "3", username = "a_bot", bot = true)
    )

    private val members = mapOf(
        "guild-1" to mapOf("2" to Member(nick = "Robert The Builder"))
    )

    private fun matches(query: String, target: String) =
        FinderMatching.fuzzyMatch(query, target)

    private fun rank(query: String): List<FinderResult.UserResult> = users.mapNotNull { user ->
        if (user.bot == true) return@mapNotNull null
        val aliases = FinderMatching.aliases(user, members)
        val score = FinderMatching.rank(user, aliases, query) ?: return@mapNotNull null
        FinderResult.UserResult(user, score, aliases.firstOrNull { it != user.username })
    }.sortedByDescending { it.score }

    private fun recipientOf(channel: Channel) =
        FinderMatching.dmRecipient(channel, users.associateBy { it.id })

    @Test
    fun subsequenceMatchesAreAccepted() {
        assertEquals(true, matches("als", "Alice Smith"))
        assertEquals(true, matches("asm", "Alice Smith"))
        assertEquals(false, matches("zzz", "Alice Smith"))
    }

    @Test
    fun aSpaceInTheQueryAlsoMatchesADash() {
        assertEquals(true, matches("bob of", "bob-offline"))
    }

    @Test
    fun aUsernameMatchOutranksANicknameMatch() {
        val ranked = rank("bob")
        assertEquals("bob-offline", ranked.first().user.username)
    }

    @Test
    fun nicknamesAreSearchable() {
        val ranked = rank("robert")
        assertEquals(1, ranked.size)
        assertEquals("Robert The Builder", ranked.first().nickname)
    }

    @Test
    fun anOfflineMemberIsFoundByTheirCachedName() {
        val ranked = rank("bob-offline")
        assertEquals(1, ranked.size)
        assertEquals("2", ranked.first().user.id)
    }

    @Test
    fun botsAreNeverSuggested() {
        assertEquals(0, rank("bot").size)
    }

    @Test
    fun aDmPrefersTheEmbeddedRecipient() {
        val channel = Channel(id = "dm1", type = 1, recipients = listOf(User(id = "9", username = "embedded")))
        assertEquals("embedded", recipientOf(channel)?.username)
    }

    @Test
    fun aDmFallsBackToRecipientIds() {
        val channel = Channel(id = "dm2", type = 1, recipient_ids = listOf("1"))
        assertEquals("alice", recipientOf(channel)?.username)
    }

    @Test
    fun aGuildChannelHasNoDmRecipient() {
        val channel = Channel(id = "c1", guild_id = "guild-1", type = 0, recipient_ids = listOf("1"))
        assertEquals(null, recipientOf(channel))
    }
}