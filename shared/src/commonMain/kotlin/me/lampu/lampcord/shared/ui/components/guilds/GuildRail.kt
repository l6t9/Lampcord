package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import me.lampu.lampcord.shared.gateway.GatewayManager
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.baseplates.RegularGuildItem
import me.lampu.lampcord.shared.ui.components.ExpressiveTooltip
import me.lampu.lampcord.shared.ui.components.tooltipText
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun GuildRail(
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    gatewayManager: GatewayManager = koinInject(),
    modifier: Modifier = Modifier
) {
    val railScrollState = rememberLazyListState()
    val guilds by guildStore.guilds.collectAsState()
    val userSettings = settingsStore.userSettings

    LazyColumn(
        state = railScrollState,
        modifier = modifier
            .width(72.dp)
            .fillMaxHeight(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
        contentPadding = PaddingValues(vertical = 8.dp)
    ) {
        item {
            val isHomeSelected = navigationStore.selectedGuild == null
            ExpressiveTooltip(
                anchorPosition = TooltipAnchorPosition.End,
                content = tooltipText("Direct Messages"),
                anchor = {
                    RegularGuildItem(
                        isSelected = isHomeSelected,
                        onClick = { navigationStore.selectHome() },
                        selectedColor = MaterialTheme.colorScheme.primary,
                        unselectedColor = MaterialTheme.colorScheme.surfaceVariant
                    ) {
                        Icon(
                            imageVector = Icons.Brand.Discord,
                            contentDescription = "Home",
                            tint = if (isHomeSelected) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(35.dp)
                        )
                    }
                }
            )
        }

        item {
            HorizontalDivider(
                modifier = Modifier
                    .width(32.dp)
                    .padding(vertical = 4.dp)
                    .clip(MaterialTheme.shapes.medium),
                thickness = 2.dp,
                color = MaterialTheme.colorScheme.outline,
            )
        }

        val folders = userSettings?.guild_folders ?: emptyList()

        if (folders.isEmpty()) {
            items(guilds.distinctBy { it.id }, key = { it.id }) { guild ->
                GuildIcon(
                    guild = guild,
                    isSelected = navigationStore.selectedGuild?.id == guild.id,
                    onClick = { navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) } }
                )
            }
        } else {
            items(folders) { folder ->
                val guildIds = folder.guild_ids.map { it.jsonPrimitive.contentOrNull ?: it.toString() }
                if (folder.id == null && guildIds.size == 1) {
                    val guildId = guildIds.first()
                    val guild = guilds.find { it.id == guildId }
                    if (guild != null) {
                        GuildIcon(
                            guild = guild,
                            isSelected = navigationStore.selectedGuild?.id == guild.id,
                            onClick = { navigationStore.selectGuild(guild) { gatewayManager.sendSubscription(it) } }
                        )
                    }
                } else {
                    GuildFolderItem(folder)
                }
            }
        }
    }
}
