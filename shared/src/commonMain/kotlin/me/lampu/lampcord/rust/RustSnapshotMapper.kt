package me.lampu.lampcord.rust

import me.lampu.lampcord.shared.model.Activity
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Member
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.PresenceUpdate
import me.lampu.lampcord.shared.model.User

object RustSnapshotMapper {
    fun mapGuilds(snapshot: RustSnapshot): List<Guild> = snapshot.guilds.map(::mapGuild)

    fun mapChangedGuilds(previous: RustSnapshot, current: RustSnapshot): List<Guild> =
        changedItems(previous.guilds, current.guilds, RustGuild::id).map(::mapGuild)

    fun mapChannels(snapshot: RustSnapshot): List<Channel> = snapshot.channels.map(::mapChannel)

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

    fun mapReadStateMessageIds(snapshot: RustSnapshot): Map<String, String> =
        snapshot.readStates
            .mapNotNull { state -> state.lastMessageId?.let { state.channelId.toString() to it.toString() } }
            .toMap()

    private fun mapMemberValues(members: List<RustMember>): Map<String, List<Pair<String, Member>>> =
        members
            .groupBy { it.guildId.toString() }
            .mapValues { (_, members) ->
                members.map { member ->
                    member.userId.toString() to Member(
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
                }
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
        icon = null,
    )

    private fun mapChannel(channel: RustChannel): Channel = Channel(
        id = channel.id.toString(),
        type = channel.kind.toInt(),
        guild_id = channel.guildId?.toString(),
        name = channel.name,
        parent_id = channel.parentId?.toString(),
        position = channel.position,
        last_message_id = null,
    )
}
