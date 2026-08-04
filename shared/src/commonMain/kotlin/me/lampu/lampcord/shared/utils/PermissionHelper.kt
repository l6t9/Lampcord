package me.lampu.lampcord.shared.utils

import me.lampu.lampcord.shared.model.*

enum class Permission(val value: Long) {
    CREATE_INSTANT_INVITE(1L shl 0),
    KICK_MEMBERS(1L shl 1),
    BAN_MEMBERS(1L shl 2),
    ADMINISTRATOR(1L shl 3),
    MANAGE_CHANNELS(1L shl 4),
    MANAGE_GUILD(1L shl 5),
    ADD_REACTIONS(1L shl 6),
    VIEW_AUDIT_LOG(1L shl 7),
    PRIORITY_SPEAKER(1L shl 8),
    STREAM(1L shl 9),
    VIEW_CHANNEL(1L shl 10),
    SEND_MESSAGES(1L shl 11),
    SEND_TTS_MESSAGES(1L shl 12),
    MANAGE_MESSAGES(1L shl 13),
    EMBED_LINKS(1L shl 14),
    ATTACH_FILES(1L shl 15),
    READ_MESSAGE_HISTORY(1L shl 16),
    MENTION_EVERYONE(1L shl 17),
    USE_EXTERNAL_EMOJIS(1L shl 18),
    VIEW_GUILD_INSIGHTS(1L shl 19),
    CONNECT(1L shl 20),
    SPEAK(1L shl 21),
    MUTE_MEMBERS(1L shl 22),
    DEAFEN_MEMBERS(1L shl 23),
    MOVE_MEMBERS(1L shl 24),
    USE_VAD(1L shl 25),
    CHANGE_NICKNAME(1L shl 26),
    MANAGE_NICKNAMES(1L shl 27),
    MANAGE_ROLES(1L shl 28),
    MANAGE_WEBHOOKS(1L shl 29),
    MANAGE_GUILD_EXPRESSIONS(1L shl 30),
    USE_APPLICATION_COMMANDS(1L shl 31),
    REQUEST_TO_SPEAK(1L shl 32),
    MANAGE_EVENTS(1L shl 33),
    MANAGE_THREADS(1L shl 34),
    CREATE_PUBLIC_THREADS(1L shl 35),
    CREATE_PRIVATE_THREADS(1L shl 36),
    USE_EXTERNAL_STICKERS(1L shl 37),
    SEND_MESSAGES_IN_THREADS(1L shl 38),
    USE_EMBEDDED_ACTIVITIES(1L shl 39),
    MODERATE_MEMBERS(1L shl 40),
    VIEW_CREATOR_MONETIZATION_ANALYTICS(1L shl 41),
    USE_SOUNDBOARD(1L shl 42),
    CREATE_GUILD_EXPRESSIONS(1L shl 43),
    CREATE_EVENTS(1L shl 44),
    USE_EXTERNAL_SOUNDS(1L shl 45),
    SEND_VOICE_MESSAGES(1L shl 46),
    SEND_POLLS(1L shl 49),
    USE_EXTERNAL_APPS(1L shl 50)
}

object PermissionHelper {
    fun computeBasePermissions(member: Member, guild: Guild): Long {
        if (guild.owner_id == member.user?.id) return -1L

        val everyoneRole = guild.roles.find { it.id == guild.id }
        var permissions = everyoneRole?.permissions?.toLongOrNull() ?: 0L

        member.roles.forEach { roleId ->
            guild.roles.find { it.id == roleId }?.let { role ->
                permissions = permissions or (role.permissions.toLongOrNull() ?: 0L)
            }
        }

        if ((permissions and Permission.ADMINISTRATOR.value) != 0L) return -1L

        return permissions
    }

    fun computeOverwrites(basePermissions: Long, member: Member, guild: Guild, channel: Channel): Long {
        if ((basePermissions and Permission.ADMINISTRATOR.value) != 0L) return -1L

        var permissions = basePermissions

        // Everyone overwrite
        channel.permission_overwrites?.find { it.id == guild.id }?.let { overwrite ->
            permissions = permissions and (overwrite.deny.toLongOrNull() ?: 0L).inv()
            permissions = permissions or (overwrite.allow.toLongOrNull() ?: 0L)
        }

        // Role overwrites
        var allow = 0L
        var deny = 0L
        member.roles.forEach { roleId ->
            channel.permission_overwrites?.find { it.id == roleId }?.let { overwrite ->
                allow = allow or (overwrite.allow.toLongOrNull() ?: 0L)
                deny = deny or (overwrite.deny.toLongOrNull() ?: 0L)
            }
        }
        permissions = permissions and deny.inv()
        permissions = permissions or allow

        // Member overwrite
        member.user?.let { user ->
            channel.permission_overwrites?.find { it.id == user.id }?.let { overwrite ->
                permissions = permissions and (overwrite.deny.toLongOrNull() ?: 0L).inv()
                permissions = permissions or (overwrite.allow.toLongOrNull() ?: 0L)
            }
        }

        return permissions
    }

    fun computePermissions(member: Member, guild: Guild, channel: Channel?): Long {
        val base = computeBasePermissions(member, guild)
        if (channel == null) return base
        return computeOverwrites(base, member, guild, channel)
    }

    fun hasPermission(member: Member, guild: Guild, channel: Channel?, permission: Permission): Boolean {
        val perms = computePermissions(member, guild, channel)
        return (perms and Permission.ADMINISTRATOR.value) != 0L || (perms and permission.value) != 0L
    }
}
