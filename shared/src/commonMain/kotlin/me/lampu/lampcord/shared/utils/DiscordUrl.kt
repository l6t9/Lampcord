package me.lampu.lampcord.shared.utils

object DiscordUrl {

    private val appHosts = setOf(
        "discord.com",
        "discordapp.com",
        "www.discord.com",
        "www.discordapp.com",
        "canary.discord.com",
        "canary.discordapp.com",
        "ptb.discord.com",
        "ptb.discordapp.com",
    )

    private val inviteHosts = setOf("discord.gg", "www.discord.gg", "discord.new")

    private val cdnHosts = setOf("cdn.discordapp.com", "media.discordapp.net")

    private val channelPath = Regex("^/channels/(@me|[1-9][0-9]{0,18})/([1-9][0-9]{0,18})(?:/([1-9][0-9]{0,18}))?/?$")
    private val threadPath = Regex("^/channels/([1-9][0-9]{0,18})/([1-9][0-9]{0,18})/threads/([1-9][0-9]{0,18})/?$")
    private val invitePath = Regex("^/(?:invites?)/([A-Za-z0-9-]{2,64})$")
    private val userPath = Regex("^/users/([1-9][0-9]{0,18})/?$")
    private val attachmentPath = Regex("^/(?:attachments|ephemeral-attachments)/([1-9][0-9]{0,18})/([1-9][0-9]{0,18})/([^/]+)$")
    private val inviteCode = Regex("^[A-Za-z0-9-]{2,64}$")

    sealed interface Target {
        data class Channel(
            val guildId: String?,
            val channelId: String,
            val messageId: String?
        ) : Target

        data class Thread(val guildId: String, val parentId: String, val threadId: String) : Target

        data class Invite(val code: String) : Target

        data class Attachment(val filename: String) : Target

        data class User(val userId: String) : Target
    }

    private class Origin(val host: String, val port: Int, val path: String)

    fun parse(url: String): Target? {
        val trimmed = url.trim()
        if (trimmed.isEmpty() || trimmed.length > 2048) return null
        if (trimmed.any { it == '\\' || it.isISOControl() || it.isWhitespace() }) return null

        val origin = trimmed.asOrigin()
        if (origin != null && origin.host in inviteHosts) {
            val code = origin.path.trim('/')
            if (inviteCode.matches(code)) return Target.Invite(code)
        }

        if (origin != null && origin.host in appHosts) {
            val path = origin.path
            threadPath.matchEntire(path)?.let { match ->
                return Target.Thread(match.groupValues[1], match.groupValues[2], match.groupValues[3])
            }
            channelPath.matchEntire(path)?.let { match ->
                return Target.Channel(
                    guildId = match.groupValues[1].takeUnless { it == "@me" },
                    channelId = match.groupValues[2],
                    messageId = match.groupValues[3].takeIf { it.isNotEmpty() }
                )
            }
            invitePath.matchEntire(path)?.let { match ->
                return Target.Invite(match.groupValues[1])
            }
            userPath.matchEntire(path)?.let { match ->
                return Target.User(match.groupValues[1])
            }
        }

        return parseAttachment(trimmed)
    }

    fun isDiscordUrl(url: String): Boolean = parse(url) != null

    private fun String.asOrigin(): Origin? {
        val separator = indexOf("://")
        if (separator <= 0) {
            val slash = indexOf('/')
            if (slash <= 0) return null
            val host = substring(0, slash).lowercase()
            if (host !in inviteHosts) return null
            return origin("https", host, -1, substring(slash))
        }
        val scheme = substring(0, separator).lowercase()
        val rest = substring(separator + 3)
        val authorityEnd = rest.indexOfFirst { it == '/' || it == '?' || it == '#' }
        val authority = if (authorityEnd < 0) rest else rest.substring(0, authorityEnd)
        val path = if (authorityEnd < 0) "" else rest.substring(authorityEnd)
            .substringBefore('?').substringBefore('#')

        if (scheme == "discord") {
            val path = if (path.startsWith("-/")) path.removePrefix("-/") else path
            return origin("https", "discord.com", -1, path)
        }
        if (scheme != "https" && scheme != "http") return null
        return origin(scheme, authority, -1, path)
    }

    private fun origin(scheme: String, authority: String, port: Int, path: String): Origin? {
        if (authority.isEmpty() || '@' in authority) return null
        val colon = authority.lastIndexOf(':')
        val host: String
        var resolvedPort = port
        if (colon >= 0 && authority.indexOf(':') == colon) {
            val candidate = authority.substring(colon + 1)
            if (candidate.isEmpty() || candidate.any { !it.isDigit() }) return null
            resolvedPort = candidate.toIntOrNull() ?: return null
            host = authority.substring(0, colon).lowercase()
        } else {
            host = authority.lowercase()
        }
        if (host.isEmpty()) return null
        val defaultPort = if (scheme == "https") 443 else 80
        if (resolvedPort != -1 && resolvedPort != defaultPort) return null
        return Origin(host, resolvedPort, path)
    }    private fun parseAttachment(url: String): Target.Attachment? {
        val origin = url.asOrigin() ?: return null
        if (origin.host !in cdnHosts) return null
        val raw = attachmentPath.matchEntire(origin.path) ?: return null
        val decoded = attachmentPath.matchEntire(decodePercent(origin.path)) ?: return null
        val filename = decoded.groupValues[3]
        if (filename == "." || filename == "..") return null
        if (filename.any { it == '\\' || it.isISOControl() }) return null
        if (filename.isBlank()) return null
        return Target.Attachment(filename)
    }

    private fun decodePercent(value: String): String {
        if ('%' !in value) return value
        val bytes = ArrayList<Byte>(value.length)
        var index = 0
        while (index < value.length) {
            val char = value[index]
            if (char == '%' && index + 2 < value.length) {
                val hex = value.substring(index + 1, index + 3)
                val decoded = hex.toIntOrNull(16)
                if (decoded == null) return value
                bytes.add(decoded.toByte())
                index += 3
            } else {
                bytes.add(char.code.toByte())
                index++
            }
        }
        return bytes.toByteArray().decodeToString()
    }

    private fun Char.isISOControl(): Boolean =
        this in '\u0000'..'\u001F' || this in '\u007F'..'\u009F'
}