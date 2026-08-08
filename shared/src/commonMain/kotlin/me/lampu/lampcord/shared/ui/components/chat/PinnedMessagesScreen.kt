package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PinnedMessagesScreen(
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    val channel = chatState.selectedChannel ?: return
    
    LaunchedEffect(channel.id) {
        chatState.showPinnedMessages()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Pins - #${channel.name}") },
                navigationIcon = {
                    IconButton(onClick = onDismiss) {
                        Icon(Icons.Filled.Close, null)
                    }
                }
            )
        }
    ) { padding ->
        if (chatState.pinnedMessages.isEmpty()) {
            Box(Modifier.fillMaxSize().padding(padding), contentAlignment = Alignment.Center) {
                Text("No pinned messages", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(padding),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                items(chatState.pinnedMessages, key = { it.id }) { message ->
                    MessageItem(message, chatState)
                }
            }
        }
    }
}
