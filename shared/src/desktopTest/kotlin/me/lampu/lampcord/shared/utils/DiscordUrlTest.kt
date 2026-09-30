package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.utils.DiscordUrl.Target
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class DiscordUrlTest {

    @Test
    fun parsesChannelLinks() {
        val target = DiscordUrl.parse("https://discord.com/channels/123456789/987654321/55555")
        assertEquals(
            Target.Channel("123456789", "987654321", "55555"),
            target
        )
    }

    @Test
    fun parsesChannelLinksWithoutAMessage() {
        assertEquals(
            Target.Channel("123456789", "987654321", null),
            DiscordUrl.parse("https://discord.com/channels/123456789/987654321")
        )
    }

    @Test
    fun parsesDirectMessageLinks() {
        assertEquals(
            Target.Channel(null, "987654321", null),
            DiscordUrl.parse("https://discord.com/channels/@me/987654321")
        )
    }

    @Test
    fun parsesTheDiscordAppScheme() {
        val expected = Target.Channel("123456789", "987654321", "55555")
        assertEquals(expected, DiscordUrl.parse("discord:///channels/123456789/987654321/55555"))
        assertEquals(expected, DiscordUrl.parse("discord://-/channels/123456789/987654321/55555"))
    }

    @Test
    fun parsesInvites() {
        assertEquals(Target.Invite("lampcord"), DiscordUrl.parse("https://discord.gg/lampcord"))
        assertEquals(Target.Invite("lampcord"), DiscordUrl.parse("discord.gg/lampcord"))
        assertEquals(Target.Invite("lampcord"), DiscordUrl.parse("https://discord.com/invite/lampcord"))
        assertEquals(Target.Invite("ab"), DiscordUrl.parse("https://discord.gg/ab"))
    }

    @Test
    fun parsesThreads() {
        assertEquals(
            Target.Thread("123456789", "987654321", "55555"),
            DiscordUrl.parse("https://discord.com/channels/123456789/987654321/threads/55555")
        )
    }

    @Test
    fun parsesAttachmentLabels() {
        val target = DiscordUrl.parse("https://cdn.discordapp.com/attachments/1/2/photo%20one.png")
        assertEquals(Target.Attachment("photo one.png"), target)
    }

    @Test
    fun parsesUserLinks() {
        assertEquals(Target.User("7654321"), DiscordUrl.parse("https://discord.com/users/7654321"))
    }

    @Test
    fun rejectsLookalikeOrigins() {
        val hostile = listOf(
            "https://evil-discord.com/channels/1/2/3",
            "https://discord.com.attacker.net/channels/1/2/3",
            "https://discord.com@evil.com/channels/1/2/3",
            "https://discord.com:8080/channels/1/2/3",
            // Backslashes are how parsers get tricked into swapping host and path.
            "https://discord.com\\@evil.com/channels/1/2/3",
            "javascript:alert(1)//discord.com/channels/1/2/3",
            "file:///channels/1/2/3",
        )
        hostile.forEach { assertNull(DiscordUrl.parse(it), "should reject: $it") }
    }

    @Test
    fun rejectsMalformedChannelPaths() {
        val bad = listOf(
            "https://discord.com/channels/0123/2/3",
            "https://discord.com/channels/1/2/3/4/5",
            "https://discord.com/channels/1/2/extra",
            "https://discord.com/channels//2/3",
        )
        bad.forEach { assertNull(DiscordUrl.parse(it), "should reject: $it") }
    }

    @Test
    fun ignoresTheGuildIdQueryParameter() {
        assertEquals(
            Target.Channel("111", "222", "333"),
            DiscordUrl.parse("https://discord.com/channels/111/222/333?guild_id=999")
        )
    }

    @Test
    fun rejectsNonDiscordUrls() {
        assertNull(DiscordUrl.parse("https://example.com/channels/1/2/3"))
        assertNull(DiscordUrl.parse("https://google.com"))
        assertNull(DiscordUrl.parse(""))
        assertNull(DiscordUrl.parse("   "))
    }

    @Test
    fun rejectsAttachmentTraversalAndControlCharacters() {
        assertNull(DiscordUrl.parse("https://cdn.discordapp.com/attachments/1/2/.."))
        assertNull(DiscordUrl.parse("https://cdn.discordapp.com/attachments/1/2/%2e%2e"))
        assertNull(DiscordUrl.parse("https://cdn.discordapp.com/attachments/1/2/a%00b.png"))
        assertNull(DiscordUrl.parse("https://cdn.discordapp.com/attachments/1/2/a%0Ab.png"))
    }

    @Test
    fun allowsTheDefaultPorts() {
        assertTrue(DiscordUrl.parse("https://discord.com:443/channels/1/2/3") != null)
        assertTrue(DiscordUrl.parse("http://discord.com:80/channels/1/2/3") != null)
    }

    @Test
    fun classifiesAttachmentsBeforeChannels() {
        assertIs<Target.Attachment>(
            DiscordUrl.parse("https://cdn.discordapp.com/attachments/1/2/file.txt")
        )
    }
}