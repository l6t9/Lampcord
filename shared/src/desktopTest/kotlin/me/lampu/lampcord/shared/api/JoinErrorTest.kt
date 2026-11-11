package me.lampu.lampcord.shared.api

import io.ktor.http.HttpStatusCode
import kotlin.test.Test
import kotlin.test.assertEquals

class JoinErrorTest {

    @Test
    fun `the server limit is mapped`() {
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.ServerLimit),
            joinFailureFor(30001, "Maximum number of guilds reached (100)", HttpStatusCode.BadRequest),
        )
    }

    @Test
    fun `join restrictions and invite errors are mapped`() {
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.JoinRestricted),
            joinFailureFor(340015, null, HttpStatusCode.Forbidden),
        )
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.InvalidInvite),
            joinFailureFor(10006, "Unknown Invite", HttpStatusCode.NotFound),
        )
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.Banned),
            joinFailureFor(40007, "You are banned from this guild.", HttpStatusCode.Forbidden),
        )
    }

    @Test
    fun `unknown errors fall back to the discord message and status`() {
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.ServerMessage, "Something specific happened"),
            joinFailureFor(99999, "Something specific happened", HttpStatusCode.BadRequest),
        )
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.RateLimited),
            joinFailureFor(null, null, HttpStatusCode.TooManyRequests),
        )
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.DiscordUnavailable, "502"),
            joinFailureFor(null, null, HttpStatusCode.BadGateway),
        )
        assertEquals(
            JoinGuildResult.Failed(JoinFailure.Rejected, "418"),
            joinFailureFor(null, null, HttpStatusCode.fromValue(418)),
        )
    }
}
