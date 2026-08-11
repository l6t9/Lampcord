package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PinnedMessagesScreen(
    onDismiss: () -> Unit,
    navigationStore: NavigationStore = koinInject(),
    messageStore: MessageStore = koinInject()
) {
    val channel = navigationStore.selectedChannel ?: return
    
    LaunchedEffect(channel.id) {
        messageStore.showPinnedMessages()
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
        if (messageStore.pinnedMessages.isEmpty()) {
            Box(Modifier.fillMaxSize().padding(padding), contentAlignment = Alignment.Center) {
                Text("No pinned messages", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(padding),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                items(messageStore.pinnedMessages, key = { it.id }) { message ->
                    MessageItem(message)
                }
            }
        }
    }
}
