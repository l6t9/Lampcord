package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.text.appendInlineContent
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.utils.DiscordUrl

        const val DISCORD_LINK_ANNOTATION = "DISCORD_LINK"

object DiscordLinkResolver {
    const val FILE_ICON = "file"
    const val MESSAGE_ICON = "message"

    fun iconKey(value: String) = "$DISCORD_LINK_ANNOTATION:$value"

    fun resolve(
        url: String,
        guilds: List<Guild>,
        channels: Map<String, Channel>
    ): DiscordLinkLabel? {
        return when (val target = DiscordUrl.parse(url) ?: return null) {
            is DiscordUrl.Target.Attachment ->
                DiscordLinkLabel(target.filename, isFile = true)

            is DiscordUrl.Target.Channel -> {
                val channel = channels[target.channelId] ?: return null
                val name = channel.name?.takeIf { it.isNotBlank() } ?: return null
                val isDm = channel.guild_id == null || channel.type == 1 || channel.type == 3

                if (isDm) {
                    DiscordLinkLabel("@$name", isMessage = target.messageId != null)
                } else {
                    val guild = guilds.firstOrNull { it.id == channel.guild_id } ?: return null
                    val guildName = guild.name?.takeIf { it.isNotBlank() } ?: return null
                    DiscordLinkLabel(
                        label = "$guildName \u203a ${channel.qualifiedName(channels)}",
                        guildIconUrl = CdnUrls.getGuildIconUrl(guild.id, guild.icon, 64),
                        isMessage = target.messageId != null
                    )
                }
            }

            is DiscordUrl.Target.Thread -> {
                val parent = channels[target.parentId] ?: return null
                val thread = channels[target.threadId] ?: return null
                val guild = guilds.firstOrNull { it.id == target.guildId } ?: return null
                val guildName = guild.name?.takeIf { it.isNotBlank() } ?: return null
                val parentName = parent.name?.takeIf { it.isNotBlank() } ?: return null
                val threadName = thread.name?.takeIf { it.isNotBlank() } ?: return null
                DiscordLinkLabel(
                    label = "$guildName \u203a $parentName \u203a $threadName",
                    guildIconUrl = CdnUrls.getGuildIconUrl(guild.id, guild.icon, 64)
                )
            }
            is DiscordUrl.Target.Invite -> null
            is DiscordUrl.Target.User -> null
        }
    }
}data class DiscordLinkLabel(
    val label: String,
    val guildIconUrl: String? = null,
    val isMessage: Boolean = false,
    val isFile: Boolean = false
)private fun Channel.qualifiedName(channels: Map<String, Channel>): String {
    val own = name?.takeIf { it.isNotBlank() } ?: return ""
    val parent = parent_id?.let { channels[it] }?.name?.takeIf { it.isNotBlank() }
    return if (parent != null) "$parent / $own" else own
}fun AnnotatedString.Builder.appendResolvedLink(
    url: String,
    resolved: DiscordLinkLabel?,
    primaryColor: Color
) {
    if (resolved == null) {
        withStyle(SpanStyle(color = primaryColor, textDecoration = TextDecoration.Underline)) {
            pushStringAnnotation("URL", url)
            append(url)
            pop()
        }
        return
    }

    withStyle(
        SpanStyle(
            color = primaryColor,
            fontWeight = FontWeight.Bold,
            background = primaryColor.copy(alpha = 0.12f)
        )
    ) {
        pushStringAnnotation("URL", url)
        resolved.guildIconUrl?.let { icon ->
            pushStringAnnotation(DISCORD_LINK_ANNOTATION, icon)
            appendInlineContent(DiscordLinkResolver.iconKey(icon))
            pop()
            append(" ")
        }
        if (resolved.isFile) {
            pushStringAnnotation(DISCORD_LINK_ANNOTATION, DiscordLinkResolver.FILE_ICON)
            appendInlineContent(DiscordLinkResolver.iconKey(DiscordLinkResolver.FILE_ICON))
            pop()
            append(" ")
        }
        append(resolved.label)
        if (resolved.isMessage) {
            append(" \u203a ")
            pushStringAnnotation(DISCORD_LINK_ANNOTATION, DiscordLinkResolver.MESSAGE_ICON)
            appendInlineContent(DiscordLinkResolver.iconKey(DiscordLinkResolver.MESSAGE_ICON))
            pop()
        }
        pop()
    }
}