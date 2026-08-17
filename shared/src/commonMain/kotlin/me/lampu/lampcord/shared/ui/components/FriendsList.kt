package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Relationship
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.state.PresenceStore
import me.lampu.lampcord.shared.state.RelationshipStore
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun FriendsList(
    relationshipStore: RelationshipStore = koinInject(),
    userStore: UserStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    presenceStore: PresenceStore = koinInject()
) {
    var selectedTab by remember { mutableIntStateOf(0) }
    val tabs = listOf("Online", "All", "Pending", "Blocked", "Add Friend")
    
    val scrollState = androidx.compose.foundation.lazy.rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }
    val relationships by relationshipStore.relationships.collectAsState()
    val currentUser by userStore.currentUser.collectAsState()
    val userSettings = settingsStore.userSettings

    val filteredRelationships = remember(relationships, selectedTab, currentUser, userSettings) {
        when (selectedTab) {
            0 -> relationships.filter { 
                val userId = it.user?.id ?: it.user_id ?: it.id
                userId != null && presenceStore.getUserStatus(userId, currentUser?.id, userSettings?.status) != "offline" && it.type == 1 
            }
            1 -> relationships.filter { it.type == 1 }
            2 -> relationships.filter { it.type == 3 || it.type == 4 } // Incoming/Outgoing
            3 -> relationships.filter { it.type == 2 } // Blocked
            else -> emptyList()
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .statusBarsPadding()
            .navigationBarsPadding()
            .pointerInput(Unit) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent()
                        when (event.type) {
                            PointerEventType.Enter -> isHovered = true
                            PointerEventType.Exit -> isHovered = false
                        }
                    }
                }
            }
    ) {
        // Tab Row
        PrimaryTabRow(
            selectedTabIndex = selectedTab,
            containerColor = Color.Transparent,
            contentColor = MaterialTheme.colorScheme.primary,
            divider = {}
        ) {
            tabs.forEachIndexed { index, title ->
                Tab(
                    selected = selectedTab == index,
                    onClick = { selectedTab = index },
                    text = { 
                        Text(
                            title, 
                            style = MaterialTheme.typography.labelLarge,
                            fontWeight = if (selectedTab == index) FontWeight.Bold else FontWeight.Medium
                        ) 
                    }
                )
            }
        }

        Box(modifier = Modifier.weight(1f)) {
            if (selectedTab == 4) {
                AddFriendUI()
            } else if (filteredRelationships.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            imageVector = Icons.Filled.Person,
                            contentDescription = null,
                            modifier = Modifier.size(64.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.2f)
                        )
                        Spacer(Modifier.height(16.dp))
                        Text(
                            text = "No friends here yet.",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        )
                    }
                }
            } else {
                LazyColumn(
                    state = scrollState,
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(filteredRelationships) { relationship ->
                        FriendItem(relationship)
                    }
                }

                VerticalScrollbar(
                    state = scrollState,
                    modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
                    isVisible = isHovered
                )
            }
        }
    }
}

@Composable
fun FriendItem(
    relationship: Relationship,
    userStore: UserStore = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    relationshipStore: RelationshipStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val user = relationship.user ?: return
    val currentUser by userStore.currentUser.collectAsState()
    val userSettings = settingsStore.userSettings
    
    val status = presenceStore.getUserStatus(user.id, currentUser?.id, userSettings?.status)
    val relationshipType = relationship.type

    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(modifier = Modifier.size(40.dp)) {
                AsyncImage(
                    model = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=128" },
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize().clip(CircleShape)
                )
                StatusIndicator(
                    status = status,
                    size = 14.dp,
                    modifier = Modifier.align(Alignment.BottomEnd),
                    borderColor = MaterialTheme.colorScheme.surface
                )
            }
            
            Spacer(Modifier.width(16.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = user.global_name ?: user.username ?: "Unknown User",
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = status.capitalize(),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            when (relationshipType) {
                2 -> {
                    TextButton(onClick = { relationshipStore.unblockUser(user.id) }) {
                        Text("Unblock")
                    }
                }
                3 -> {
                    Button(
                        onClick = { relationshipStore.addFriend(user.id) },
                        shape = RoundedCornerShape(8.dp)
                    ) {
                        Text("Accept")
                    }
                    Spacer(Modifier.width(8.dp))
                    TextButton(onClick = { relationshipStore.removeFriend(user.id) }) {
                        Text("Ignore")
                    }
                }
                4 -> {
                    TextButton(onClick = { relationshipStore.removeFriend(user.id) }) {
                        Text("Cancel")
                    }
                }
                else -> {
                    Row {
                        IconButton(onClick = { navigationStore.openDm(user.id) }) {
                            Icon(Icons.Filled.Chat, "Message", modifier = Modifier.size(20.dp))
                        }
                        ContextMenu(
                            items = listOf(
                                ContextMenuItem("Remove Friend", Icons.Filled.PersonRemove) {
                                    relationshipStore.removeFriend(user.id)
                                },
                                ContextMenuItem("Block", Icons.Filled.Block, color = Color.Red) {
                                    relationshipStore.blockUser(user.id)
                                }
                            )
                        ) {
                            IconButton(onClick = {}) {
                                Icon(Icons.Filled.MoreVert, "More", modifier = Modifier.size(20.dp))
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun AddFriendUI(relationshipStore: RelationshipStore = koinInject()) {
    var query by remember { mutableStateOf("") }
    var statusText by remember { mutableStateOf<String?>(null) }
    
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
    ) {
        Text(
            "Add Friend",
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold
        )
        Text(
            "You can add friends with their Discord username.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        
        Spacer(Modifier.height(16.dp))
        
        OutlinedTextField(
            value = query,
            onValueChange = { query = it },
            modifier = Modifier.fillMaxWidth(),
            placeholder = { Text("Enter a username") },
            trailingIcon = {
                Button(
                    onClick = {
                        val name = query.trim()
                        if (name.isBlank()) return@Button
                        relationshipStore.sendFriendRequest(name, null) { success ->
                            statusText = if (success) "Friend request sent!" else "Couldn't find that user."
                        }
                    },
                    enabled = query.isNotBlank(),
                    modifier = Modifier.padding(end = 8.dp),
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Text("Send Friend Request")
                }
            }
        )
        
        statusText?.let {
            Spacer(Modifier.height(12.dp))
            Text(
                text = it,
                style = MaterialTheme.typography.bodyMedium,
                color = if (it == "Friend request sent!") MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error
            )
        }
    }
}

private fun String.capitalize() = replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
