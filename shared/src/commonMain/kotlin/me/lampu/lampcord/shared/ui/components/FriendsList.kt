package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Relationship
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun FriendsList(chatState: ChatState) {
    var selectedTab by remember { mutableIntStateOf(0) }
    val tabs = listOf("Online", "All", "Pending", "Blocked", "Add Friend")
    
    val scrollState = androidx.compose.foundation.lazy.rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }
    val filteredRelationships = remember(chatState.relationships, selectedTab) {
        when (selectedTab) {
            0 -> chatState.relationships.filter { 
                val userId = it.user?.id ?: it.user_id ?: it.id
                userId != null && chatState.getUserStatus(userId) != "offline" && it.type == 1 
            }
            1 -> chatState.relationships.filter { it.type == 1 }
            2 -> chatState.relationships.filter { it.type == 3 || it.type == 4 } // Incoming/Outgoing
            3 -> chatState.relationships.filter { it.type == 2 } // Blocked
            else -> emptyList()
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
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
        Surface(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 0.dp
        ) {
            Row(
                modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Filled.Person,
                    contentDescription = null,
                    modifier = Modifier.size(24.dp),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(Modifier.width(12.dp))
                Text(
                    text = "Friends",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold
                )
                Spacer(Modifier.width(24.dp))
                VerticalDivider(modifier = Modifier.height(24.dp).padding(horizontal = 8.dp), color = MaterialTheme.colorScheme.outlineVariant)
                
                tabs.forEachIndexed { index, tab ->
                    val isSelected = selectedTab == index
                    val isAddFriend = index == 4
                    
                    Surface(
                        onClick = { selectedTab = index },
                        modifier = Modifier.padding(horizontal = 8.dp),
                        shape = RoundedCornerShape(4.dp),
                        color = when {
                            isAddFriend && isSelected -> MaterialTheme.colorScheme.primary
                            isAddFriend -> Color.Transparent
                            isSelected -> MaterialTheme.colorScheme.surfaceVariant
                            else -> Color.Transparent
                        }
                    ) {
                        Text(
                            text = tab,
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                            style = MaterialTheme.typography.labelLarge,
                            fontWeight = FontWeight.Medium,
                            color = when {
                                isAddFriend && isSelected -> MaterialTheme.colorScheme.onPrimary
                                isAddFriend -> Color(0xFF43B581) // Discord Green
                                isSelected -> MaterialTheme.colorScheme.onSurface
                                else -> MaterialTheme.colorScheme.onSurfaceVariant
                            }
                        )
                    }
                }
            }
        }

        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

        if (selectedTab == 4) {
            AddFriendUI(chatState)
        } else {
            Box(modifier = Modifier.fillMaxSize()) {
                LazyColumn(
                    state = scrollState,
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    item {
                        Text(
                            text = "${tabs[selectedTab]} — ${filteredRelationships.size}".uppercase(),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            fontWeight = FontWeight.Bold,
                            modifier = Modifier.padding(bottom = 8.dp)
                        )
                    }
                    items(filteredRelationships, key = { it.id ?: (it.user?.id ?: "") }) { relationship ->
                        FriendItem(relationship, chatState)
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
fun FriendItem(relationship: Relationship, chatState: ChatState) {
    val user = relationship.user ?: return
    val status = chatState.getUserStatus(user.id)
    val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=128" }

    Surface(
        onClick = { /* TODO: Select DM */ },
        modifier = Modifier.fillMaxWidth().height(64.dp).padding(horizontal = 4.dp),
        shape = RoundedCornerShape(8.dp),
        color = Color.Transparent
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(modifier = Modifier.size(38.dp)) {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    shape = CircleShape,
                    color = MaterialTheme.colorScheme.secondaryContainer
                ) {
                    if (avatarUrl != null) {
                        AsyncImage(model = avatarUrl, contentDescription = user.username, modifier = Modifier.fillMaxSize())
                    } else {
                        Box(contentAlignment = Alignment.Center) {
                            Text(user.username?.take(1)?.uppercase() ?: "?", style = MaterialTheme.typography.titleMedium)
                        }
                    }
                }
                StatusIndicator(
                    status = status,
                    size = 14.dp,
                    modifier = Modifier.align(Alignment.BottomEnd).offset(x = 2.dp, y = 2.dp),
                    borderColor = MaterialTheme.colorScheme.surface,
                    backgroundColor = MaterialTheme.colorScheme.surface
                )
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = user.global_name ?: user.username ?: "Unknown User",
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                    if (user.global_name != null) {
                        Spacer(Modifier.width(4.dp))
                        Text(
                            text = user.username ?: "",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
                Text(
                    text = status.replaceFirstChar { it.uppercase() },
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                IconButton(onClick = { /* Message */ }) {
                    Icon(Icons.Filled.Chat, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                IconButton(onClick = { /* More */ }) {
                    Icon(Icons.Filled.MoreVert, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
fun AddFriendUI(chatState: ChatState) {
    var text by remember { mutableStateOf("") }
    
    Column(modifier = Modifier.padding(16.dp)) {
        Text("ADD FRIEND", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
        Text(
            "You can add friends with their Discord username.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Spacer(Modifier.height(16.dp))
        
        OutlinedTextField(
            value = text,
            onValueChange = { text = it },
            modifier = Modifier.fillMaxWidth(),
            placeholder = { Text("Enter a Username", style = MaterialTheme.typography.bodyMedium) },
            trailingIcon = {
                Button(
                    onClick = { /* TODO */ },
                    enabled = text.isNotBlank(),
                    shape = RoundedCornerShape(4.dp),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 0.dp),
                    modifier = Modifier.padding(end = 8.dp).height(32.dp)
                ) {
                    Text("Send Friend Request", style = MaterialTheme.typography.labelMedium)
                }
            },
            colors = OutlinedTextFieldDefaults.colors(
                focusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                focusedBorderColor = MaterialTheme.colorScheme.primary,
                unfocusedBorderColor = Color.Transparent
            ),
            shape = RoundedCornerShape(8.dp),
            singleLine = true
        )
    }
}
