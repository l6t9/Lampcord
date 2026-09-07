package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Invite
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.guilds.GuildProfileSheet
import org.koin.compose.koinInject

@Composable
fun InviteEmbedView(
    code: String,
    guildApi: GuildApi = koinInject(),
    guildStore: GuildStore = koinInject()
) {
    var invite by remember { mutableStateOf<Invite?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    var showProfile by remember { mutableStateOf(false) }

    LaunchedEffect(code) {
        invite = guildApi.resolveInvite(code)
        isLoading = false
    }

    if (isLoading) {
        Surface(
            modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 400.dp).fillMaxWidth(),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            shape = RoundedCornerShape(8.dp)
        ) {
            Box(Modifier.height(80.dp), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(modifier = Modifier.size(24.dp))
            }
        }
        return
    }

    val data = invite ?: return
    val guild = data.guild ?: return
    val isMember = guildStore.guilds.value.any { it.id == guild.id }

    Surface(
        onClick = { showProfile = true },
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 400.dp).fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(8.dp),
        tonalElevation = 1.dp
    ) {
        Column {
            if (guild.banner != null) {
                AsyncImage(
                    model = "https://cdn.discordapp.com/banners/${guild.id}/${guild.banner}.png?size=600",
                    contentDescription = null,
                    modifier = Modifier.fillMaxWidth().height(100.dp),
                    contentScale = ContentScale.Crop
                )
            }

            Row(
                modifier = Modifier.padding(12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=128" }
                if (iconUrl != null) {
                    AsyncImage(
                        model = iconUrl,
                        contentDescription = null,
                        modifier = Modifier.size(50.dp).clip(RoundedCornerShape(12.dp))
                    )
                } else {
                    Box(
                        modifier = Modifier.size(50.dp).background(MaterialTheme.colorScheme.secondaryContainer, RoundedCornerShape(12.dp)),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(guild.name?.take(1) ?: "?", style = MaterialTheme.typography.titleLarge)
                    }
                }

                Spacer(Modifier.width(12.dp))

                Column(modifier = Modifier.weight(1f)) {
                    val inviteHeader = when {
                        isMember -> "YOU ARE A MEMBER"
                        data.inviter != null -> "${data.inviter.global_name ?: data.inviter.username} INVITED YOU TO JOIN"
                        else -> "YOU WERE INVITED TO JOIN"
                    }
                    Text(
                        text = inviteHeader.uppercase(),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis
                    )
                    Text(
                        text = guild.name ?: "Server",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFF23A559), CircleShape))
                        Spacer(Modifier.width(4.dp))
                        Text(
                            "${data.approximate_presence_count ?: 0} Online",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(Modifier.width(8.dp))
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFFB5BAC1), CircleShape))
                        Spacer(Modifier.width(4.dp))
                        Text(
                            "${data.approximate_member_count ?: 0} Members",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                Button(
                    onClick = { showProfile = true },
                    shape = RoundedCornerShape(8.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = if (isMember) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.primary,
                        contentColor = if (isMember) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onPrimary
                    )
                ) {
                    Text(if (isMember) "Joined" else "Join")
                }
            }
        }
    }

    if (showProfile) {
        GuildProfileSheet(
            guildId = guild.id,
            onDismiss = { showProfile = false }
        )
    }
}
