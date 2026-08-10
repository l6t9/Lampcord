package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.EmojiIndex

class AutocompleteStore(
    private val memberListStore: MemberListStore,
    private val relationshipStore: RelationshipStore,
    private val guildStore: GuildStore,
    private val commandStore: CommandStore
) {
    var autocompleteType by mutableStateOf<AutocompleteType?>(null)
    var autocompleteQuery by mutableStateOf("")
    var autocompleteSelectedIndex by mutableStateOf(0)
    val autocompleteItems = mutableStateListOf<AutocompleteItem>()

    fun updateAutocomplete(type: AutocompleteType?, query: String, selectedGuild: Guild?) {
        if (type == null) {
            autocompleteType = null
            autocompleteQuery = ""
            autocompleteItems.clear()
            return
        }

        autocompleteType = type
        autocompleteQuery = query
        
        val results = mutableListOf<AutocompleteItem>()
        when (type) {
            AutocompleteType.MENTION, AutocompleteType.USER -> {
                val guildId = selectedGuild?.id
                if (type == AutocompleteType.MENTION) {
                    if (query.isEmpty() || "everyone".contains(query, ignoreCase = true)) {
                        results.add(AutocompleteItem(id = "everyone", title = "everyone", replacement = "@everyone", searchReplacement = "everyone", iconType = Icons.Filled.Group))
                    }
                    if (query.isEmpty() || "here".contains(query, ignoreCase = true)) {
                        results.add(AutocompleteItem(id = "here", title = "here", replacement = "@here", searchReplacement = "here", iconType = Icons.Filled.Group))
                    }
                }

                val members = if (guildId != null) {
                     memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.filter {
                         val name = it.nick ?: it.user?.global_name ?: it.user?.username ?: ""
                         name.contains(query, ignoreCase = true) || it.user?.username?.contains(query, ignoreCase = true) == true
                     }.take(10)
                } else {
                    relationshipStore.relationships.filter { 
                        it.user?.global_name?.contains(query, ignoreCase = true) == true || 
                        it.user?.username?.contains(query, ignoreCase = true) == true
                    }.mapNotNull { it.user?.let { u -> Member(user = u) } }.take(10)
                }
                
                results.addAll(members.map { member ->
                    val user = member.user!!
                    val name = member.nick ?: user.global_name ?: user.username ?: "Unknown User"
                    AutocompleteItem(
                        id = user.id,
                        title = name,
                        subtitle = user.username,
                        icon = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" },
                        replacement = "<@${user.id}>",
                        searchReplacement = user.username
                    )
                })

                if (type == AutocompleteType.MENTION) {
                    val roles = selectedGuild?.roles?.filter { 
                        it.name.contains(query, ignoreCase = true) 
                    }?.take(5) ?: emptyList()
                    results.addAll(roles.map { role ->
                        AutocompleteItem(
                            id = role.id,
                            title = role.name,
                            iconType = Icons.Filled.Group,
                            replacement = "<@&${role.id}>",
                            searchReplacement = role.name,
                            color = if (role.color != 0) Color(role.color or 0xFF000000.toInt()) else null
                        )
                    })
                }
            }
            AutocompleteType.CHANNEL -> {
                val channels = guildStore.channels.filter { 
                    it.type in listOf(0, 2, 4, 5, 13, 15, 16) && it.name?.contains(query, ignoreCase = true) == true 
                }.take(10)
                results.addAll(channels.map { channel ->
                    AutocompleteItem(
                        id = channel.id,
                        title = channel.name ?: "unnamed",
                        iconType = when (channel.type) {
                            4 -> Icons.Filled.Folder
                            15 -> Icons.Rounded.Forum
                            2, 13 -> Icons.AutoMirrored.Filled.VolumeUp
                            5 -> Icons.Filled.Campaign
                            else -> Icons.Filled.Tag
                        },
                        replacement = if (channel.type == 4) channel.name ?: "" else "<#${channel.id}>",
                        searchReplacement = channel.name ?: "unnamed"
                    )
                })
            }
            AutocompleteType.COMMAND -> {
                val commands = commandStore.availableCommands.filter { 
                    it.name.contains(query, ignoreCase = true) 
                }.take(10)
                results.addAll(commands.map { command ->
                    val app = commandStore.availableApplications.find { it.id == command.application_id }
                    AutocompleteItem(
                        id = command.id,
                        title = "/${command.name}",
                        subtitle = command.description,
                        icon = app?.icon?.let { "https://cdn.discordapp.com/app-icons/${app.id}/$it.png?size=64" },
                        replacement = command.name,
                        isCommand = true,
                        commandObj = command
                    )
                })
            }
            AutocompleteType.EMOJI -> {
                val emojis = selectedGuild?.emojis?.filter { 
                    it.name?.contains(query, ignoreCase = true) == true 
                }?.take(15) ?: emptyList()
                results.addAll(emojis.map { emoji ->
                    AutocompleteItem(
                        id = emoji.id ?: emoji.name ?: "",
                        title = ":${emoji.name}:",
                        icon = if (emoji.id != null) "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=64" else null,
                        replacement = if (emoji.id != null) "<:${emoji.name}:${emoji.id}>" else ":${emoji.name}:"
                    )
                })

                // Add standard emojis
                if (results.size < 20) {
                    val standardEmojis = EmojiIndex.getAllEmojis().filter {
                        it.name?.contains(query, ignoreCase = true) == true
                    }.take(20 - results.size)
                    results.addAll(standardEmojis.map { emoji ->
                        AutocompleteItem(
                            id = emoji.name ?: "",
                            title = ":${emoji.name}:",
                            icon = emoji.url,
                            replacement = EmojiIndex.getCharForName(emoji.name ?: "") ?: emoji.name ?: ""
                        )
                    })
                }
            }
            AutocompleteType.ROLE -> { }
        }

        autocompleteItems.clear()
        autocompleteItems.addAll(results)
        autocompleteSelectedIndex = 0
    }

    fun clear() {
        autocompleteType = null
        autocompleteQuery = ""
        autocompleteItems.clear()
        autocompleteSelectedIndex = 0
    }
}
