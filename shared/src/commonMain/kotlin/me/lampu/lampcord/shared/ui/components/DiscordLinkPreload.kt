package me.lampu.lampcord.shared.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlinx.coroutines.withContext
import me.lampu.lampcord.shared.api.ChannelApi
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.state.EntityStore
import me.lampu.lampcord.shared.utils.DiscordUrl
import org.koin.compose.koinInject

private val DISCORD_URL_REGEX = Regex(
    """discord(?:app)?\.com/[^\s<>()\[\]\u201c\u201d"']+|discord://[^\s<>()\[\]\u201c\u201d"']+""",
    RegexOption.IGNORE_CASE
)

@Composable
fun rememberDiscordLinkPreload(
    content: String,
    entityStore: EntityStore = koinInject(),
    channelApi: ChannelApi = koinInject(),
    guildApi: GuildApi = koinInject()
) {
    val targets = remember(content) { content.discordTargets() }
    LaunchedEffect(targets) {
        if (targets.isEmpty()) return@LaunchedEffect
        withContext(Dispatchers.IO) {
            targets.forEach { target ->
                when (target) {
                    is DiscordUrl.Target.Channel -> {
                        if (!entityStore.channels.value.containsKey(target.channelId)) {
                            runCatching { channelApi.getChannel(target.channelId) }.getOrNull()
                                ?.let { entityStore.updateChannel(it) }
                        }
                        target.guildId?.let { loadGuild(it, entityStore, guildApi) }
                    }

                    is DiscordUrl.Target.Thread -> {
                        listOf(target.parentId, target.threadId).forEach { channelId ->
                            if (!entityStore.channels.value.containsKey(channelId)) {
                                runCatching { channelApi.getChannel(channelId) }.getOrNull()
                                    ?.let { entityStore.updateChannel(it) }
                            }
                        }
                        loadGuild(target.guildId, entityStore, guildApi)
                    }

                    else -> Unit
                }
            }
        }
    }
}

private suspend fun loadGuild(guildId: String, entityStore: EntityStore, guildApi: GuildApi) {
    if (entityStore.guilds.value.containsKey(guildId)) return
    runCatching { guildApi.getGuild(guildId) }.getOrNull()?.let { entityStore.updateGuild(it) }
}

private fun String.discordTargets(): List<DiscordUrl.Target> {
    if (!contains("discord", ignoreCase = true)) return emptyList()
    return DISCORD_URL_REGEX.findAll(this)
        .map { DiscordUrl.parse(it.value) }
        .filterIsInstance<DiscordUrl.Target>()
        .toList()
}