package me.lampu.lampcord.rust

import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.AvatarDecorationData
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.MemberListGroup
import me.lampu.lampcord.shared.model.MemberListListItem
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.model.Relationship
import me.lampu.lampcord.shared.model.User

object RustSnapshotMapper {
    fun mapGuilds(snapshot: RustSnapshot): List<Guild> = snapshot.guilds.map(::mapGuild)

    fun mapChannels(snapshot: RustSnapshot): List<Channel> = snapshot.channels.map(::mapChannel)

    fun mapPrivateChannels(snapshot: RustSnapshot): List<Channel> =
        snapshot.channels.filter { it.guildId == null }.map(::mapChannel)

    fun mapChangedChannels(previous: RustSnapshot, current: RustSnapshot): List<Channel> =
        changedItems(previous.channels, current.channels, RustChannel::id).map(::mapChannel)

    fun mapMessages(snapshot: RustSnapshot): List<Message> = snapshot.messages.map(::mapMessage)

    fun mapChangedMessages(previous: RustSnapshot, current: RustSnapshot): List<Message> =
        changedItems(previous.messages, current.messages, RustMessage::id).map(::mapMessage)

    fun mapMessage(message: RustMessage): Message = Message(
        id = message.id.toString(),
        channel_id = message.channelId.toString(),
        author = User(
            id = (message.authorId ?: 0uL).toString(),
            username = message.authorName,
            global_name = message.authorName,
            avatar = message.authorAvatar,
        ),
        content = message.content,
        timestamp = message.timestamp,
        edited_timestamp = message.editedTimestamp,
    )

    fun mapMembers(snapshot: RustSnapshot): Map<String, List<Pair<String, Member>>> =
        mapMemberValues(snapshot.members)

    fun mapChangedMembers(
        previous: RustSnapshot,
        current: RustSnapshot,
    ): Map<String, List<Pair<String, Member>>> = mapMemberValues(
        changedItems(previous.members, current.members) { member ->
            member.guildId to member.userId
        },
    )

    fun mapPresences(snapshot: RustSnapshot): List<PresenceUpdate> =
        snapshot.presences.map(::mapPresence)

    fun mapChangedPresences(previous: RustSnapshot, current: RustSnapshot): List<PresenceUpdate> =
        changedItems(previous.presences, current.presences, RustPresence::userId).map(::mapPresence)

    fun mapAcknowledgements(snapshot: RustSnapshot): Map<String, String> =
        acknowledgements(snapshot.readStates)

    fun mapChangedAcknowledgements(previous: RustSnapshot, current: RustSnapshot): Map<String, String> =
        acknowledgements(changedItems(previous.readStates, current.readStates, RustReadState::channelId))

    fun mapMemberListItems(snapshot: RustSnapshot): List<MemberListListItem?> =
        snapshot.memberList.rows.map { row ->
            when (row.kind) {
                ROW_GROUP -> MemberListListItem(
                    group = MemberListGroup(id = row.groupId.orEmpty(), count = row.groupCount),
                )
                ROW_MEMBER -> row.member?.let { MemberListListItem(member = toMember(it)) }
                else -> null
            }
        }

    private fun toMember(member: RustMember): Member = Member(
        user = User(
            id = member.userId.toString(),
            username = member.username,
            avatar = member.avatar,
        ),
        nick = member.nickname,
        avatar = member.avatar,
        roles = member.roles.map(ULong::toString),
        presence = member.status?.let { status ->
            PresenceUpdate(
                user = User(id = member.userId.toString()),
                status = status,
            )
        },
    )

    fun mapRelationships(snapshot: RustSnapshot): List<Relationship> =
        snapshot.relationships.map { relationship ->
            val id = relationship.userId.toString()
            Relationship(
                id = id,
                type = relationship.kind,
                user = User(
                    id = id,
                    username = relationship.username,
                    global_name = relationship.globalName,
                    avatar = relationship.avatar,
                    avatar_decoration_data = relationship.avatarDecoration?.let { AvatarDecorationData(asset = it) },
                ),
                user_id = id,
            )
        }

    private const val ROW_GROUP = 1
    private const val ROW_MEMBER = 2

    private fun mapMemberValues(members: List<RustMember>): Map<String, List<Pair<String, Member>>> =
        members
            .groupBy { it.guildId.toString() }
            .mapValues { (_, members) ->
                members.map { member -> member.userId.toString() to toMember(member) }
            }

    private fun mapPresence(presence: RustPresence): PresenceUpdate = PresenceUpdate(
        user = User(id = presence.userId.toString()),
        user_id = presence.userId.toString(),
        status = presence.status,
        activities = presence.activities.map { name -> Activity(name = name, type = 0) },
    )

    private fun acknowledgements(readStates: List<RustReadState>): Map<String, String> =
        readStates
            .mapNotNull { state -> state.lastReadMessageId?.let { state.channelId.toString() to it.toString() } }
            .toMap()

    private fun <T, K> changedItems(
        previous: List<T>,
        current: List<T>,
        key: (T) -> K,
    ): List<T> {
        if (previous.isEmpty()) return current
        val previousById = previous.associateBy(key)
        return current.filter { item -> previousById[key(item)] != item }
    }

    private fun mapGuild(guild: RustGuild): Guild = Guild(
        id = guild.id.toString(),
        name = guild.name,
        icon = guild.iconUrl,
    )

    private fun mapChannel(channel: RustChannel): Channel = Channel(
        id = channel.id.toString(),
        type = channel.kind.toInt(),
        guild_id = channel.guildId?.toString(),
        name = channel.name,
        parent_id = channel.parentId?.toString(),
        position = channel.position,
        last_message_id = null,
        icon = channel.icon,
        recipients = channel.recipients.map { recipient ->
            User(
                id = recipient.id.toString(),
                username = recipient.username,
                global_name = recipient.globalName,
                avatar = recipient.avatar,
            )
        },
    )
}
