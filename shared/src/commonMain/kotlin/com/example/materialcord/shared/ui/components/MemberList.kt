package com.example.materialcord.shared.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.Member
import com.example.materialcord.shared.model.MemberListGroup
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.icons.MaterialcordIcons

@Composable
fun MemberList(chatState: ChatState) {
    Column(modifier = Modifier.fillMaxSize()) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            shape = RoundedCornerShape(0.dp)
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(top = 8.dp),
                contentPadding = PaddingValues(bottom = 80.dp)
            ) {
                items(chatState.memberListItems) { item ->
                    when {
                        item?.member != null -> MemberItem(item.member, chatState)
                        item?.group != null -> MemberGroupItem(item.group, chatState)
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MemberItem(member: Member, chatState: ChatState) {
    val user = member.user ?: return
    val avatarUrl = member.avatar?.let {
        "https://cdn.discordapp.com/guilds/${chatState.selectedGuild?.id}/users/${user.id}/avatars/$it.png"
    } ?: user.avatar?.let {
        "https://cdn.discordapp.com/avatars/${user.id}/$it.png"
    }

    val roleColor = remember(member.roles, chatState.selectedGuild) {
        val guild = chatState.selectedGuild ?: return@remember Color.Unspecified
        val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
        val highestRole = memberRoles.maxByOrNull { it.position }
        if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.Unspecified
    }

    val contextMenuItems = listOf(
        ContextMenuItem("Profile", MaterialcordIcons.Filled.AccountCircle) { chatState.showProfile(user.id) },
        ContextMenuItem("Mention", MaterialcordIcons.Filled.Add) { /* TODO */ },
        ContextMenuItem("Message", MaterialcordIcons.Filled.Share) { /* TODO */ }
    )

    MaterialcordContextMenu(items = contextMenuItems, modifier = Modifier.fillMaxWidth()) {
        Surface(
            modifier = Modifier.fillMaxWidth().height(44.dp).padding(horizontal = 8.dp),
            onClick = { chatState.showProfile(user.id) },
            color = Color.Transparent,
            shape = MaterialTheme.shapes.small
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Surface(
                    modifier = Modifier.size(32.dp),
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.secondaryContainer
                ) {
                    if (avatarUrl != null) {
                        AsyncImage(model = avatarUrl, contentDescription = user.username, modifier = Modifier.fillMaxSize())
                    } else {
                        Box(contentAlignment = Alignment.Center) {
                            Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.labelMedium)
                        }
                    }
                }
                Spacer(modifier = Modifier.width(12.dp))
                Column {
                    Text(
                        text = member.nick ?: user.global_name ?: user.username,
                        style = MaterialTheme.typography.bodyMedium,
                        color = if (roleColor != Color.Unspecified) roleColor else MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }
    }
}

@Composable
fun MemberGroupItem(group: MemberListGroup, chatState: ChatState) {
    val roleName = remember(group.id, chatState.selectedGuild) {
        if (group.id == "online") "Online"
        else if (group.id == "offline") "Offline"
        else chatState.selectedGuild?.roles?.find { it.id == group.id }?.name ?: group.id
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
            .padding(top = 8.dp),
        contentAlignment = Alignment.CenterStart
    ) {
        Text(
            text = "${roleName.uppercase()} — ${group.count ?: 0}",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontWeight = FontWeight.Bold
        )
    }
}
