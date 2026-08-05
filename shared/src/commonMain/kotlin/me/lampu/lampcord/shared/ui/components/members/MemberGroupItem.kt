package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.MemberListGroup
import me.lampu.lampcord.shared.state.ChatState

@Composable
fun MemberGroupItem(group: MemberListGroup, chatState: ChatState) {
    val role = remember(group.id, chatState.selectedGuild) {
        chatState.selectedGuild?.roles?.find { it.id == group.id }
    }
    val roleName = remember(group.id, role) {
        if (group.id == "online") "Online"
        else if (group.id == "offline") "Offline"
        else role?.name ?: group.id
    }
    
    // Find up-to-date count from chatState.memberListGroups if the item's count is stale
    val displayCount = remember(group, chatState.memberListGroups.size) {
        chatState.memberListGroups.find { it.id == group.id }?.let { it.count ?: it.member_count } ?: group.count ?: group.member_count ?: 0
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
            .padding(top = 8.dp),
        contentAlignment = Alignment.CenterStart
    ) {
        Text(
            text = "$roleName — $displayCount",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontWeight = FontWeight.Bold,
            letterSpacing = 0.5.sp
        )
    }
}
