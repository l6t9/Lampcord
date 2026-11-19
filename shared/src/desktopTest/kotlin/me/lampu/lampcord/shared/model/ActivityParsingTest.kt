package me.lampu.lampcord.shared.model

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ActivityParsingTest {
    private val strictJson = Json { ignoreUnknownKeys = true }

    @Test
    fun `decodes documented activity fields and gateway buttons`() {
        val presence =
            strictJson.decodeFromString<PresenceUpdate>(
                """
                {
                  "status": "online",
                  "activities": [{
                    "id": "activity-1",
                    "name": "Visual Studio Code",
                    "type": 0,
                    "created_at": "1695164784863",
                    "session_id": "session-1",
                    "platform": "desktop",
                    "application_id": "123456789",
                    "details": "Editing MainActivity.kt",
                    "state": "Lampcord",
                    "status_display_type": 2,
                    "timestamps": { "start": "1695164700000", "end": 1695164800000 },
                    "party": { "id": "party-1", "size": [2, 4], "privacy": 1 },
                    "assets": {
                      "large_image": "large",
                      "large_url": "https://cdn.example/large.png",
                      "small_image": "small",
                      "small_url": "https://cdn.example/small.png"
                    },
                    "buttons": ["View Repository", "Documentation"],
                    "metadata": {
                      "button_urls": ["https://github.com/example/repo", "https://docs.example/activity"]
                    }
                  }],
                  "hidden_activities": [{ "name": "Hidden", "type": 0 }],
                  "has_played_game": true,
                  "client_status": { "desktop": "online", "embedded": "online", "vr": "idle" }
                }
                """.trimIndent()
            )

        val activity = presence.activities.single()
        assertEquals("activity-1", activity.id)
        assertEquals("session-1", activity.session_id)
        assertEquals("desktop", activity.platform)
        assertEquals(1695164784863L, activity.created_at)
        assertEquals(1695164700000L, activity.timestamps?.start)
        assertEquals(1695164800000L, activity.timestamps?.end)
        assertEquals(2, activity.status_display_type)
        assertEquals(listOf(2, 4), activity.party?.size)
        assertEquals(1, activity.party?.privacy)
        assertEquals("https://cdn.example/large.png", activity.assets?.large_url)
        assertEquals("Hidden", presence.hidden_activities.single().name)
        assertEquals(true, presence.has_played_game)
        assertEquals("online", presence.client_status.embedded)
        assertEquals("idle", presence.client_status.vr)
        assertEquals(
            listOf(
                ActivityButton("View Repository", "https://github.com/example/repo"),
                ActivityButton("Documentation", "https://docs.example/activity"),
            ),
            activity.resolvedButtons,
        )
    }

    @Test
    fun `resolves RPC button objects and ignores unsafe urls`() {
        val activity =
            strictJson.decodeFromString<Activity>(
                """
                {
                  "name": "Chess",
                  "type": 0,
                  "buttons": [
                    { "label": "Watch", "url": "https://example.com/watch" },
                    { "label": "Join", "url": "javascript:alert(1)" }
                  ]
                }
                """.trimIndent()
            )

        assertEquals(listOf(ActivityButton("Watch", "https://example.com/watch")), activity.resolvedButtons)
    }

    @Test
    fun `outbound presence normalizes buttons and drops receive only fields`() {
        val json =
            Json {
                ignoreUnknownKeys = true
                explicitNulls = false
            }
        val activity =
            Activity(
                id = "receive-only",
                name = "Game",
                type = 0,
                created_at = 42L,
                session_id = "session-1",
                platform = "desktop",
                details_url = "https://example.com/details",
                buttons =
                    listOf(
                        kotlinx.serialization.json.JsonPrimitive("Docs"),
                        buildJsonObject {
                            put("label", "RPC")
                            put("url", "https://rpc.example/invite")
                        },
                    ),
                metadata =
                    ActivityMetadata(
                        button_urls = listOf("https://docs.example/activity"),
                        type = "game",
                    ),
            )

        val payload = buildUpdatePresencePayload("online", listOf(activity), json)
        val outgoing = payload["activities"]!!.jsonArray.single().jsonObject

        assertNull(outgoing["id"])
        assertNull(outgoing["created_at"])
        assertNull(outgoing["session_id"])
        assertNull(outgoing["platform"])
        assertNull(outgoing["details_url"])
        assertEquals("online", payload["status"]?.jsonPrimitive?.content)
        assertEquals(listOf("Docs", "RPC"), outgoing["buttons"]!!.jsonArray.map { it.jsonPrimitive.content })
        assertEquals(
            listOf("https://docs.example/activity", "https://rpc.example/invite"),
            outgoing["metadata"]!!.jsonObject["button_urls"]!!.jsonArray.map { it.jsonPrimitive.content },
        )
        assertEquals("game", outgoing["metadata"]!!.jsonObject["type"]?.jsonPrimitive?.content)
    }
}
