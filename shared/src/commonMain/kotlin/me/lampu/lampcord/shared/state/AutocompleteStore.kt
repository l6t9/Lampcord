package me.lampu.lampcord.shared.state

import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.EmojiIndex
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class AutocompleteStore(
    private val memberListStore: MemberListStore,
    private val relationshipStore: RelationshipStore,
    private val guildStore: GuildStore,
    private val userStore: UserStore,
    private val commandStore: CommandStore,
    private val guildApi: GuildApi
) {
    private val scope = CoroutineScope(Dispatchers.Main)
    private var memberSearchJob: Job? = null

    var autocompleteType by mutableStateOf<AutocompleteType?>(null)
    var autocompleteQuery by mutableStateOf("")
    var autocompleteSelectedIndex by mutableStateOf(0)
    val autocompleteItems = mutableStateListOf<AutocompleteItem>()
    
    var searchAutocompleteType by mutableStateOf<AutocompleteType?>(null)
    var searchAutocompleteQuery by mutableStateOf("")
    var searchAutocompleteSelectedIndex by mutableStateOf(0)
    val searchAutocompleteItems = mutableStateListOf<AutocompleteItem>()

    fun updateAutocomplete(type: AutocompleteType?, query: String, selectedGuild: Guild?, selectedChannel: Channel? = null, isSearch: Boolean = false) {
        memberSearchJob?.cancel()
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
                    val local = memberListStore.memberListItems.filterNotNull().mapNotNull { it.member }.filter { member ->
                        val name = member.nick ?: member.user?.global_name ?: member.user?.username ?: ""
                        name.contains(query, ignoreCase = true) || member.user?.username?.contains(query, ignoreCase = true) == true
                    }.take(10).toMutableList()

                    if (local.size < 5 && query.isNotEmpty()) {
                        memberSearchJob = scope.launch {
                            val remote = guildApi.searchGuildMembers(guildId, query, limit = 10)
                            if (remote.isNotEmpty()) {
                                var changed = false
                                remote.forEach { rm ->
                                    if (local.none { it.user?.id == rm.user?.id }) {
                                        local.add(rm)
                                        changed = true
                                    }
                                }
                                if (changed) {
                                    updateAutocompleteResults(type, query, selectedGuild, isSearch, local)
                                }
                            }
                        }
                    }
                    local
                } else if (selectedChannel != null && selectedChannel.guild_id == null) {
                    val dmUsers = mutableListOf<User>()
                    selectedChannel.recipients?.let { dmUsers.addAll(it) }
                    if (selectedChannel.recipients.isNullOrEmpty() && !selectedChannel.recipient_ids.isNullOrEmpty()) {
                        selectedChannel.recipient_ids.forEach { id ->
                            userStore.getUser(id)?.let { dmUsers.add(it) }
                        }
                    }
                    userStore.currentUser.value?.let { dmUsers.add(it) }

                    val allMembers = dmUsers.distinctBy { it.id }.filter { user ->
                        query.isEmpty() ||
                                user.username?.contains(query, ignoreCase = true) == true ||
                                user.global_name?.contains(query, ignoreCase = true) == true ||
                                user.id == query
                    }.map { Member(user = it) }
                    allMembers.take(10)
                } else {
                    relationshipStore.relationships.value.filter { rel ->
                        val user = rel.user ?: rel.user_id?.let { id -> User(id = id) }
                        user?.global_name?.contains(query, ignoreCase = true) == true || 
                        user?.username?.contains(query, ignoreCase = true) == true
                    }.mapNotNull { rel -> rel.user?.let { u -> Member(user = u) } }.take(10)
                }
                
                addMembersToResults(members, results, selectedGuild, isSearch)

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
                }.sortedWith(compareBy(
                    { !(it.name?.equals(query, ignoreCase = true) == true) },
                    { !(it.name?.startsWith(query, ignoreCase = true) == true) },
                    { it.position ?: 0 }
                )).take(15)
                
                results.addAll(channels.map { channel ->
                    val categoryName = channel.parent_id?.let { pid ->
                        guildStore.allGuildChannels.value[pid]?.name
                    }
                    AutocompleteItem(
                        id = channel.id,
                        title = channel.name ?: "unnamed",
                        subtitle = categoryName,
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
                        guildEmojis.addAll(guild.emojis.map { it.copy(guild_id = guild.id) })
                    }
                } else {
                    selectedGuild?.emojis?.let { emojis ->
                        guildEmojis.addAll(emojis.map { it.copy(guild_id = selectedGuild.id) })
                    }
                }

                val customMatches = guildEmojis.filter { emo ->
                    emo.name?.contains(query, ignoreCase = true) == true
                }.distinctBy { it.id }.sortedWith(compareBy(
                    { !(it.name?.equals(query, ignoreCase = true) == true) },
                    { !(it.name?.startsWith(query, ignoreCase = true) == true) },
                    { it.guild_id != selectedGuild?.id },
                    { it.name }
                ))

                val standardMatches = EmojiIndex.getAllEmojis().filter { emo ->
                    emo.name?.contains(query, ignoreCase = true) == true
                }.sortedWith(compareBy(
                    { !(it.name?.equals(query, ignoreCase = true) == true) },
                    { !(it.name?.startsWith(query, ignoreCase = true) == true) },
                    { it.name }
                ))

                val allMatches = (customMatches.map { it to true } + standardMatches.map { it to false })
                    .take(30)

                val nameCounts = mutableMapOf<String, Int>()
                
                results.addAll(allMatches.map { (emoji, isCustom) ->
                    val rawName = emoji.name ?: "emoji"
                    val count = nameCounts[rawName] ?: 0
                    nameCounts[rawName] = count + 1
                    val disambiguatedName = if (count > 0) "$rawName-$count" else rawName
                    
                    if (isCustom) {
                        val isExternal = emoji.guild_id != null && emoji.guild_id != selectedGuild?.id
                        val isAnimated = emoji.animated == true
                        val needsNitro = isExternal || isAnimated
                        
                        val replacement = if (needsNitro && !hasNitro && freeNitro) {
                            if (realmojis) {
                                "<${if (isAnimated) "a" else ""}:F_$rawName:${emoji.id}>"
                            } else {
                                "https://cdn.discordapp.com/emojis/${emoji.id}.${if (isAnimated) "gif" else "png"}?size=48&name=$rawName"
                            }
                        } else {
                            "<${if (isAnimated) "a" else ""}:$rawName:${emoji.id}>"
                        }

                        AutocompleteItem(
                            id = emoji.id ?: disambiguatedName,
                            title = ":$disambiguatedName:",
                            subtitle = if (isExternal) guildStore.guilds.value.find { it.id == emoji.guild_id }?.name else null,
                            icon = if (emoji.id != null) "https://cdn.discordapp.com/emojis/${emoji.id}.png?size=64" else null,
                            replacement = replacement,
                            inputText = ":$disambiguatedName:"
                        )
                    } else {
                        AutocompleteItem(
                            id = emoji.name ?: "",
                            title = ":$disambiguatedName:",
                            icon = emoji.url,
                            replacement = EmojiIndex.getCharForName(emoji.name ?: "") ?: emoji.name ?: "",
                            inputText = ":$disambiguatedName:"
                        )
                    }
                })
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

    private fun addMembersToResults(
        members: List<Member>,
        results: MutableList<AutocompleteItem>,
        selectedGuild: Guild?,
        isSearch: Boolean
    ) {
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
    }

    private fun updateAutocompleteResults(
        type: AutocompleteType,
        query: String,
        selectedGuild: Guild?,
        isSearch: Boolean,
        members: List<Member>
    ) {
        val newResults = mutableListOf<AutocompleteItem>()
        if (type == AutocompleteType.MENTION) {
            if (query.isEmpty() || "everyone".contains(query, ignoreCase = true)) {
                newResults.add(AutocompleteItem(id = "everyone", title = "everyone", replacement = "@everyone", searchReplacement = "everyone", iconType = Icons.Filled.Group, inputText = "@everyone"))
            }
            if (query.isEmpty() || "here".contains(query, ignoreCase = true)) {
                newResults.add(AutocompleteItem(id = "here", title = "here", replacement = "@here", searchReplacement = "here", iconType = Icons.Filled.Group, inputText = "@here"))
            }
        }
        
        addMembersToResults(members, newResults, selectedGuild, isSearch)
        
        if (type == AutocompleteType.MENTION) {
            val roles = selectedGuild?.roles?.filter { role ->
                role.name.contains(query, ignoreCase = true)
            }?.take(5) ?: emptyList()
            newResults.addAll(roles.map { role ->
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

        if (isSearch) {
            searchAutocompleteItems.clear()
            searchAutocompleteItems.addAll(newResults)
        } else {
            autocompleteItems.clear()
            autocompleteItems.addAll(newResults)
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
