package me.lampu.lampcord.rust

import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.User
import uniffi.lampcord_ffi.ChannelSnapshot
import uniffi.lampcord_ffi.ClientSnapshot
import uniffi.lampcord_ffi.GuildSnapshot
import uniffi.lampcord_ffi.MessageSnapshot

/**
 * Read-only parity slice: map Rust FFI snapshots to existing Kotlin UI models.
 * This does not replace Kotlin stores/gateway; it proves Rust core data can feed the UI.
 */
object RustSnapshotMapper {

    fun mapGuilds(snapshot: ClientSnapshot): List<Guild> = snapshot.guilds.map(::mapGuild)

    fun mapChannels(snapshot: ClientSnapshot): List<Channel> = snapshot.channels.map(::mapChannel)

    fun mapMessages(snapshot: ClientSnapshot): List<Message> = snapshot.messages.map(::mapMessage)

    fun mapGuild(g: GuildSnapshot): Guild = Guild(
        id = g.id.toString(),
        name = g.name,
        // Rust snapshot currently provides icon_url; Kotlin model expects icon hash.
        icon = null,
    )

    fun mapChannel(c: ChannelSnapshot): Channel = Channel(
        id = c.id.toString(),
        type = c.kind.toInt(),
        guild_id = c.guildId?.toString(),
        name = c.name,
        parent_id = c.parentId?.toString(),
        position = c.position,
    )

    fun mapMessage(m: MessageSnapshot): Message = Message(
        id = m.id.toString(),
        channel_id = m.channelId.toString(),
        author = m.authorId?.let { id ->
            User(
                id = id.toString(),
                username = m.authorName,
                global_name = m.authorName,
            )
        } ?: User(id = "0", username = m.authorName, global_name = m.authorName),
        content = m.content,
        timestamp = m.timestamp,
        edited_timestamp = m.editedTimestamp,
    )
}
