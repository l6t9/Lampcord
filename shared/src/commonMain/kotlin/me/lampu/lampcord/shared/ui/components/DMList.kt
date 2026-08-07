package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState

import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun DMList(chatState: ChatState) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }

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
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
                .padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "Direct Messages",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.weight(1f)
            )

            IconButton(
                onClick = { chatState.selectFriends() },
                modifier = Modifier.size(32.dp)
            ) {
                Icon(
                    imageVector = if (chatState.isFriendsSelected) Icons.Filled.Person else Icons.Outlined.Person,
                    contentDescription = "Friends",
                    tint = if (chatState.isFriendsSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp)
                )
            }
        }
        
        Box(modifier = Modifier.fillMaxSize()) {
            LazyColumn(
                state = scrollState,
                modifier = Modifier.fillMaxSize().padding(top = 8.dp),
                contentPadding = PaddingValues(bottom = 68.dp)
            ) {
                if (chatState.privateChannels.isEmpty()) {
                    items(10) {
                        DMSkeleton()
                    }
                } else {
                    items(chatState.privateChannels, key = { it.id }) { channel ->
                        DMItem(channel, chatState)
                    }
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

@Composable
fun DMSkeleton() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        ShimmerBox(
            modifier = Modifier.size(32.dp),
            shape = androidx.compose.foundation.shape.CircleShape
        )
        Spacer(modifier = Modifier.width(12.dp))
        ShimmerBox(
            modifier = Modifier
                .width(100.dp)
                .height(14.dp),
            shape = RoundedCornerShape(7.dp)
        )
    }
}
