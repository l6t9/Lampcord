package me.lampu.lampcord.shared.ui.components.members

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
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
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.components.RoleIcon
import org.koin.compose.koinInject

@Composable
fun MemberGroupItem(
    group: MemberListGroup,
    navigationStore: NavigationStore = koinInject(),
    memberListStore: MemberListStore = koinInject()
) {
    val role = remember(group.id, navigationStore.selectedGuild) {
        navigationStore.selectedGuild?.roles?.find { it.id == group.id }
    }
    val roleName = remember(group.id, role) {
        when (group.id) {
            "online" -> "Online"
            "offline" -> "Offline"
            else -> role?.name ?: "Loading..."
        }
    }
    
    // Find up-to-date count from memberListStore.memberListGroups if the item's count is stale
    val displayCount = remember(group, memberListStore.memberListGroups.size) {
        val currentGroup = memberListStore.memberListGroups[group.id]
        if (currentGroup != null) {
            currentGroup.count ?: currentGroup.member_count
        } else {
            group.count ?: group.member_count
        } ?: 0
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp)
            .padding(top = 8.dp),
        contentAlignment = Alignment.CenterStart
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (role != null && (role.icon != null || role.unicode_emoji != null)) {
                RoleIcon(role, size = 16.dp)
                Spacer(Modifier.width(8.dp))
            }
            Text(
                text = "$roleName — $displayCount",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontWeight = FontWeight.Bold,
                letterSpacing = 0.5.sp
            )
        }
    }
}
