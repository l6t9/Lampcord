package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.getPlatformName
import org.koin.compose.koinInject
import kotlinx.serialization.json.*
import kotlinx.serialization.encodeToString

@Composable
fun NavigationSettings(
    onBack: () -> Unit,
    settingsStore: SettingsStore = koinInject()
) {
    SettingsSubScreen(
        title = "Navigation",
        onNavigateBack = onBack,
        contentScrollable = true
    ) {
        NavigationSettingsContent(settingsStore)
    }
}

@Composable
private fun NavigationSettingsContent(
    settingsStore: SettingsStore
) {
    val isMobile = remember { getPlatformName() == "android" || getPlatformName() == "ios" }
    if (!isMobile) return

    val json = Json { ignoreUnknownKeys = true }
    
    var items by remember(settingsStore.navTabsOrderJson) {
        val order = runCatching { 
            json.decodeFromString<List<String>>(settingsStore.navTabsOrderJson)
        }.getOrDefault(listOf("home", "friends", "search", "mentions", "settings"))
        mutableStateOf(order)
    }

    fun updateOrder(newList: List<String>) {
        items = newList
        settingsStore.navTabsOrderJson = json.encodeToString(newList)
    }

    Column(modifier = Modifier.fillMaxSize()) {
        Material3SettingsGroup(
            title = "Tabs Visibility",
            items = listOf(
                switchSettingsItem(
                    title = "Home",
                    checked = settingsStore.showNavHome,
                    onCheckedChange = { settingsStore.showNavHome = it }
                ),
                switchSettingsItem(
                    title = "Friends",
                    checked = settingsStore.showNavFriends,
                    onCheckedChange = { settingsStore.showNavFriends = it }
                ),
                switchSettingsItem(
                    title = "Search",
                    checked = settingsStore.showNavSearch,
                    onCheckedChange = { settingsStore.showNavSearch = it }
                ),
                switchSettingsItem(
                    title = "Mentions",
                    checked = settingsStore.showNavMentions,
                    onCheckedChange = { settingsStore.showNavMentions = it }
                ),
                switchSettingsItem(
                    title = "You",
                    checked = settingsStore.showNavSettings,
                    onCheckedChange = { settingsStore.showNavSettings = it }
                ),
                switchSettingsItem(
                    title = "Hide Navigation Labels",
                    description = "Only show icons in the navigation bar.",
                    checked = settingsStore.hideNavLabels,
                    onCheckedChange = { settingsStore.hideNavLabels = it }
                )
            )
        )

        Spacer(Modifier.height(16.dp))

        Text(
            text = "Tab Order",
            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Medium),
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)
        )

        Column(
            modifier = Modifier
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            items.forEachIndexed { index, item ->
                val label = when (item) {
                    "home" -> "Home"
                    "friends" -> "Friends"
                    "search" -> "Search"
                    "mentions" -> "Mentions"
                    "settings" -> "You"
                    else -> item
                }
                val icon = when (item) {
                    "home" -> Icons.Brand.Discord
                    "friends" -> Icons.Rounded.Person
                    "search" -> Icons.Filled.Search
                    "mentions" -> Icons.Rounded.AlternateEmail
                    "settings" -> Icons.Filled.Settings
                    else -> Icons.Filled.QuestionMark
                }

                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    color = MaterialTheme.colorScheme.surfaceContainer
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(icon, null, modifier = Modifier.size(24.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        Spacer(Modifier.width(16.dp))
                        Text(label, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.weight(1f))
                        
                        Row {
                            IconButton(
                                onClick = {
                                    if (index > 0) {
                                        val newList = items.toMutableList()
                                        val temp = newList[index]
                                        newList[index] = newList[index - 1]
                                        newList[index - 1] = temp
                                        updateOrder(newList)
                                    }
                                },
                                enabled = index > 0
                            ) {
                                Icon(Icons.Rounded.ArrowUpward, null)
                            }
                            IconButton(
                                onClick = {
                                    if (index < items.size - 1) {
                                        val newList = items.toMutableList()
                                        val temp = newList[index]
                                        newList[index] = newList[index + 1]
                                        newList[index + 1] = temp
                                        updateOrder(newList)
                                    }
                                },
                                enabled = index < items.size - 1
                            ) {
                                Icon(Icons.Rounded.ArrowDownward, null)
                            }
                        }
                    }
                }
            }
        }
        
        Spacer(Modifier.height(16.dp))
    }
}
