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
    private val userStore: UserStore,
    private val commandStore: CommandStore
) {
    var autocompleteType by mutableStateOf<AutocompleteType?>(null)
    var autocompleteQuery by mutableStateOf("")
    var autocompleteSelectedIndex by mutableStateOf(0)
    val autocompleteItems = mutableStateListOf<AutocompleteItem>()
    
    var searchAutocompleteType by mutableStateOf<AutocompleteType?>(null)
    var searchAutocompleteQuery by mutableStateOf("")
    var searchAutocompleteSelectedIndex by mutableStateOf(0)
    val searchAutocompleteItems = mutableStateListOf<AutocompleteItem>()

    fun updateAutocomplete(type: AutocompleteType?, query: String, selectedGuild: Guild?, selectedChannel: Channel? = null, isSearch: Boolean = false) {
        if (type == null) {
            if (isSearch) {
                searchAutocompleteType = null
                searchAutocompleteQuery = ""
                searchAutocompleteItems.clear()
            } else {
                autocompleteType = null
                autocompleteQuery = ""
                autocompleteItems.clear()
            }
            return
        }

        if (isSearch) {
            searchAutocompleteType = type
            searchAutocompleteQuery = query
        } else {
            autocompleteType = type
            autocompleteQuery = query
        }
        
        val results = mutableListOf<AutocompleteItem>()
        when (type) {
            AutocompleteType.MENTION, AutocompleteType.USER -> {
                val guildId = selectedGuild?.id
                if (type == AutocompleteType.MENTION) {
                    if (query.isEmpty() || "everyone".contains(query, ignoreCase = true)) {
                        results.add(AutocompleteItem(id = "everyone", title = "everyone", replacement = "@everyone", searchReplacement = "everyone", iconType = Icons.Filled.Group, inputText = "@everyone"))
                    }
                    if (query.isEmpty() || "here".contains(query, ignoreCase = true)) {
                        results.add(AutocompleteItem(id = "here", title = "here", replacement = "@here", searchReplacement = "here", iconType = Icons.Filled.Group, inputText = "@here"))
                    }
                }

                val members = if (guildId != null) {
                     memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.filter { member ->
                         val name = member.nick ?: member.user?.global_name ?: member.user?.username ?: ""
                         name.contains(query, ignoreCase = true) || member.user?.username?.contains(query, ignoreCase = true) == true
                     }.take(10)
                } else if (selectedChannel != null && selectedChannel.guild_id == null) {
                    val recipients = selectedChannel.recipients?.filter { user ->
                        user.global_name?.contains(query, ignoreCase = true) == true ||
                                user.username?.contains(query, ignoreCase = true) == true
                    }?.map { Member(user = it) } ?: emptyList()

                    val allMembers = recipients.toMutableList()
                    userStore.currentUser.value?.let { currentUser ->
                        if (currentUser.global_name?.contains(query, ignoreCase = true) == true ||
                            currentUser.username?.contains(query, ignoreCase = true) == true) {
                            allMembers.add(Member(user = currentUser))
                        }
                    }
                    allMembers.take(10)
                } else {
                    relationshipStore.relationships.value.filter { rel ->
                        val user = rel.user ?: rel.user_id?.let { id -> User(id = id) }
                        user?.global_name?.contains(query, ignoreCase = true) == true || 
                        user?.username?.contains(query, ignoreCase = true) == true
                    }.mapNotNull { rel -> rel.user?.let { u -> Member(user = u) } }.take(10)
                }
                
                results.addAll(members.map { member ->
                    val user = member.user!!
                    val name = member.nick ?: user.global_name ?: user.username ?: "Unknown User"
                    
                    val roleData = if (selectedGuild != null) {
                        val colorRole = member.getRoleColorRole(selectedGuild)
                        if (colorRole != null) {
                            val primaryInt = colorRole.colors?.primary_color ?: colorRole.color
                            val gradient = if (colorRole.colors?.secondary_color != null) {
                                listOfNotNull(
                                    Color(primaryInt or 0xFF000000.toInt()),
                                    Color(colorRole.colors.secondary_color or 0xFF000000.toInt()),
                                    colorRole.colors.tertiary_color?.let { Color(it or 0xFF000000.toInt()) }
                                )
                            } else null
                            val color = if (primaryInt != 0) Color(primaryInt or 0xFF000000.toInt()) else null
                            color to gradient
                        } else null
                    } else null

                    AutocompleteItem(
                        id = user.id,
                        title = name,
                        subtitle = user.username,
                        icon = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=64" },
                        replacement = if (isSearch) user.id else "<@${user.id}>",
                        searchReplacement = user.username,
                        color = roleData?.first,
                        gradient = roleData?.second,
                        inputText = if (isSearch) user.id else "@$name"
                    )
                })

                if (type == AutocompleteType.MENTION) {
                    val roles = selectedGuild?.roles?.filter { role ->
                        role.name.contains(query, ignoreCase = true) 
                    }?.take(5) ?: emptyList()
                    results.addAll(roles.map { role ->
                        val primaryInt = role.colors?.primary_color ?: role.color
                        AutocompleteItem(
                            id = role.id,
                            title = role.name,
                            iconType = Icons.Filled.Group,
                            replacement = "<@&${role.id}>",
                            searchReplacement = role.name,
                            color = if (primaryInt != 0) Color(primaryInt or 0xFF000000.toInt()) else null,
                            gradient = if (role.colors?.secondary_color != null) {
                                listOfNotNull(
                                    Color(primaryInt or 0xFF000000.toInt()),
                                    Color(role.colors.secondary_color or 0xFF000000.toInt()),
                                    role.colors.tertiary_color?.let { Color(it or 0xFF000000.toInt()) }
                                )
                            } else null,
                            inputText = "@${role.name}"
                        )
                    })
                }
            }
            AutocompleteType.CHANNEL -> {
                val channels = guildStore.allGuildChannels.value.values.filter { channel ->
                    channel.guild_id == selectedGuild?.id && 
                    channel.type in listOf(0, 2, 4, 5, 13, 15, 16) && 
                    channel.name?.contains(query, ignoreCase = true) == true 
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
                        replacement = if (isSearch) channel.id else if (channel.type == 4) channel.name ?: "" else "<#${channel.id}>",
                        searchReplacement = channel.name ?: "unnamed",
                        inputText = if (isSearch) channel.id else "#${channel.name ?: ""}"
                    )
                })
            }
            AutocompleteType.COMMAND -> {
                val commands = commandStore.availableCommands.filter { cmd ->
                    cmd.name.contains(query, ignoreCase = true) 
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
                // Standard (unicode) emojis come from EmojiIndex, which is loaded
                // lazily. Make sure it is initialized before we filter below.
                EmojiIndex.initialize()

                val currentUser = userStore.currentUser.value
                val hasNitro = (currentUser?.premium_type ?: 0) > 0
                val freeNitro = me.lampu.lampcord.shared.settings.Settings.shared.freeNitroEmojis
                val realmojis = me.lampu.lampcord.shared.settings.Settings.shared.realmojis

                val guildEmojis = mutableListOf<Emoji>()
                if (hasNitro || freeNitro) {
                    guildStore.guilds.value.forEach { guild ->
                        guildEmojis.addAll(guild.emojis)
                    }
                } else {
                    selectedGuild?.emojis?.let { guildEmojis.addAll(it) }
                }

                val filteredEmojis = guildEmojis.filter { emo ->
                    emo.name?.contains(query, ignoreCase = true) == true 
                }.distinctBy { it.id }.sortedBy { it.guild_id != selectedGuild?.id }.take(20)

                results.addAll(filteredEmojis.map { emoji ->
                    val isExternal = emoji.guild_id != null && emoji.guild_id != selectedGuild?.id
                    val isAnimated = emoji.animated == true
                    
                    val replacement = if (emoji.id != null) {
                        val needsNitro = isExternal || isAnimated
                        if (needsNitro && !hasNitro && freeNitro) {
                            if (realmojis) {
                                "<${if (isAnimated) "a" else ""}:F_${emoji.name}:${emoji.id}>"
                            } else {
                                "https://cdn.discordapp.com/emojis/${emoji.id}.${if (isAnimated) "gif" else "png"}?size=48&name=${emoji.name}"
                            }
                        } else {
                            "<${if (isAnimated) "a" else ""}:${emoji.name}:${emoji.id}>"
                        }
                    } else ":${emoji.name}:"

                    AutocompleteItem(
                        id = emoji.id ?: emoji.name ?: "",
                        title = ":${emoji.name}:",
                        icon = if (emoji.id != null) "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=64" else null,
                        replacement = replacement,
                        inputText = ":${emoji.name}:"
                    )
                })

                // Add standard emojis
                if (results.size < 25) {
                    val standardEmojis = EmojiIndex.getAllEmojis().filter { emo ->
                        emo.name?.contains(query, ignoreCase = true) == true
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

        if (isSearch) {
            searchAutocompleteItems.clear()
            searchAutocompleteItems.addAll(results)
            searchAutocompleteSelectedIndex = 0
        } else {
            autocompleteItems.clear()
            autocompleteItems.addAll(results)
            autocompleteSelectedIndex = 0
        }
    }

    fun clear(isSearch: Boolean = false) {
        if (isSearch) {
            searchAutocompleteType = null
            searchAutocompleteQuery = ""
            searchAutocompleteItems.clear()
            searchAutocompleteSelectedIndex = 0
        } else {
            autocompleteType = null
            autocompleteQuery = ""
            autocompleteItems.clear()
            autocompleteSelectedIndex = 0
        }
    }
}
