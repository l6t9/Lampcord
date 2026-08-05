package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.components.*

@Composable
fun ProfileHeader(
    profile: UserProfile,
    chatState: ChatState,
    theme: ProfileTheme,
    isExpanded: Boolean,
    onExpand: (() -> Unit)? = null
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile

    // Avatar
    val avatarUrl = profile.guild_member?.avatar?.let {
        "https://cdn.discordapp.com/guilds/${chatState.selectedGuild?.id}/users/${user.id}/avatars/$it.png?size=160"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160"
    }

    Row(verticalAlignment = Alignment.Bottom) {
        Box(
            modifier = Modifier
                .offset(y = if (isExpanded) (-60).dp else (-45).dp)
                .size(if (isExpanded) 120.dp else 94.dp)
                .background(theme.cutoutColor, CircleShape)
                .padding(if (isExpanded) 8.dp else 6.dp)
        ) {
            Surface(
                modifier = Modifier.fillMaxSize(),
                shape = CircleShape,
                color = Color.DarkGray,
                onClick = { onExpand?.invoke() },
                enabled = !isExpanded
            ) {
                if (avatarUrl != null) {
                    AsyncImage(
                        model = avatarUrl,
                        contentDescription = null,
                        modifier = Modifier.fillMaxSize(),
                        filterQuality = FilterQuality.Medium
                    )
                } else {
                    Box(contentAlignment = Alignment.Center) {
                        Text(user.username.take(1).uppercase(), style = if (isExpanded) MaterialTheme.typography.displaySmall else MaterialTheme.typography.headlineLarge)
                    }
                }
            }
            val status = chatState.getUserStatus(user.id)
            StatusIndicator(
                status = status,
                size = if (isExpanded) 30.dp else 24.dp,
                modifier = Modifier
                    .align(Alignment.BottomEnd),
                borderColor = theme.cutoutColor,
                borderWidth = if (isExpanded) 5.dp else 4.dp,
                backgroundColor = theme.cutoutColor
            )
        }

        val presence = profile.guild_member?.presence ?: chatState.presences[user.id]
        val customStatus = presence?.activities?.find { it.type == 4 }

        if (customStatus != null) {
            UserActivity(
                activity = customStatus,
                compact = true,
                modifier = Modifier
                    .offset(y = if (isExpanded) (-50).dp else (-35).dp)
                    .padding(start = 12.dp, bottom = 8.dp)
            )
        }
    }

    Column(modifier = Modifier.offset(y = if (isExpanded) (-50).dp else (-35).dp)) {
        Text(
            text = profile.guild_member?.nick ?: user.global_name ?: user.username,
            style = if (isExpanded) MaterialTheme.typography.headlineSmall else MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.Bold,
            color = Color.White
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(user.username, style = MaterialTheme.typography.bodyMedium, color = theme.contentColor.copy(alpha = 0.9f))
            val pronouns = guildMeta?.pronouns ?: userMeta?.pronouns ?: user.pronouns
            if (!pronouns.isNullOrBlank()) {
                Text(" • $pronouns", style = MaterialTheme.typography.bodyMedium, color = theme.contentColor.copy(alpha = 0.7f), modifier = Modifier.padding(start = 4.dp))
            }
        }
        Spacer(Modifier.height(8.dp))
        UserBadges(badges = profile.badges + profile.guild_badges, flags = user.public_flags ?: 0)
    }
}
