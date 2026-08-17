@file:OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)

package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.model.Role
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AdaptiveModalBottomSheet
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsGroup
import me.lampu.lampcord.shared.ui.components.settings.Material3SettingsItem
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject

@Composable
fun ManageRolesSheet(
    profile: UserProfile,
    guild: Guild,
    onDismiss: () -> Unit,
    profileStore: ProfileStore = koinInject(),
    guildApi: GuildApi = koinInject(),
    userStore: UserStore = koinInject()
) {
    val isMobile = getPlatformName() == "android" || getPlatformName() == "ios"

    if (isMobile) {
        val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
        AdaptiveModalBottomSheet(
            onDismissRequest = onDismiss,
            sheetState = sheetState
        ) {
            ManageRolesContent(profile, guild, onDismiss, profileStore, guildApi, userStore)
        }
    } else {
        Dialog(onDismissRequest = onDismiss) {
            Card(
                modifier = Modifier.width(420.dp).heightIn(max = 560.dp),
                shape = MaterialTheme.shapes.large,
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh
                )
            ) {
                ManageRolesContent(profile, guild, onDismiss, profileStore, guildApi, userStore)
            }
        }
    }
}

@Composable
private fun ManageRolesContent(
    profile: UserProfile,
    guild: Guild,
    onDismiss: () -> Unit,
    profileStore: ProfileStore,
    guildApi: GuildApi,
    userStore: UserStore
) {
    val currentUser by userStore.currentUser.collectAsState()
    val myMember = remember(guild.id, currentUser) {
        currentUser?.id?.let { userStore.getMember(guild.id, it) }
    }
    val isOwner = guild.owner_id != null && guild.owner_id == currentUser?.id

    val myHighestPosition = remember(guild, myMember, isOwner) {
        if (isOwner) Int.MAX_VALUE
        else myMember?.roles
            ?.mapNotNull { id -> guild.roles.find { it.id == id } }
            ?.maxOfOrNull { it.position } ?: -1
    }

    fun isRoleManageable(role: Role): Boolean {
        if (role.id == guild.id) return false
        if (role.managed) return false
        if (role.position >= myHighestPosition) return false
        return true
    }

    var selectedRoles by remember(profile.user.id, guild.id) {
        mutableStateOf((profile.guild_member?.roles ?: emptyList()).toMutableSet())
    }
    val scope = rememberCoroutineScope()

    fun toggleRole(role: Role, checked: Boolean) {
        val updated = selectedRoles.toMutableSet()
        if (checked) updated.add(role.id) else updated.remove(role.id)
        selectedRoles = updated
        scope.launch {
            val ok = guildApi.modifyGuildMemberRoles(guild.id, profile.user.id, updated.toList())
            if (ok) {
                profileStore.updateMemberRoles(profile.user.id, updated.toList())
            }
        }
    }

    val roles = remember(guild) { guild.roles.sortedByDescending { it.position } }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(bottom = 24.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    text = "Manage User",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    text = guild.name.orEmpty(),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            TextButton(onClick = onDismiss) { Text("Done") }
        }

        Material3SettingsGroup(
            title = "Roles",
            items = roles.map { role ->
                val checked = role.id in selectedRoles
                val manageable = isRoleManageable(role)
                val roleColor = if (role.color != 0) Color(role.color or 0xFF000000.toInt())
                else MaterialTheme.colorScheme.onSurfaceVariant

                Material3SettingsItem(
                    leadingContent = {
                        Box(modifier = Modifier.size(12.dp).clip(CircleShape).background(roleColor))
                    },
                    title = { Text(role.name, color = roleColor) },
                    description = if (role.managed) {
                        { Text("This role is automatically managed by an integration.") }
                    } else null,
                    trailingContent = {
                        Checkbox(
                            checked = checked,
                            onCheckedChange = if (manageable) { { toggleRole(role, it) } } else null,
                            enabled = manageable
                        )
                    },
                    enabled = manageable,
                    onClick = if (manageable) { { toggleRole(role, !checked) } } else null
                )
            }
        )
    }
}
