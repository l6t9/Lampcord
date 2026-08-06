package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.Channel
import me.lampu.lampcord.shared.model.ForumTag
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun ForumPostList(chatState: ChatState) {
    val forumChannel = chatState.selectedChannel ?: return

    Column(modifier = Modifier.fillMaxSize()) {
        // Forum Header
        Surface(
            modifier = Modifier.fillMaxWidth(),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 1.dp
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = forumChannel.name ?: "Forum",
                    style = MaterialTheme.typography.headlineSmall,
                    fontWeight = FontWeight.Bold
                )
                if (!forumChannel.topic.isNullOrBlank()) {
                    Text(
                        text = forumChannel.topic,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                
                Spacer(Modifier.height(16.dp))
                
                // Tags
                val tags = forumChannel.available_tags ?: emptyList()
                if (tags.isNotEmpty()) {
                    LazyRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        items(tags) { tag ->
                            FilterChip(
                                selected = false,
                                onClick = { /* TODO: filter */ },
                                label = { Text(tag.name) },
                                leadingIcon = {
                                    if (tag.emoji_name != null) {
                                        Text(tag.emoji_name)
                                    }
                                }
                            )
                        }
                    }
                    Spacer(Modifier.height(16.dp))
                }
                
                OutlinedButton(
                    onClick = { /* TODO: New Post */ },
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Icon(Icons.Filled.Add, null)
                    Spacer(Modifier.width(8.dp))
                    Text("New Post")
                }
            }
        }

        if (chatState.isForumLoading && chatState.forumThreads.isEmpty()) {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
                contentPadding = PaddingValues(vertical = 16.dp)
            ) {
                items(10) { ForumSkeleton() }
            }
        } else if (chatState.forumThreads.isEmpty()) {
            Box(modifier = Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                Text(
                    text = "No posts yet",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                verticalArrangement = Arrangement.spacedBy(1.dp), // Discord uses small gaps
                contentPadding = PaddingValues(vertical = 16.dp)
            ) {
                items(chatState.forumThreads, key = { it.id }) { thread ->
                    ForumPostItem(thread, forumChannel, onClick = { chatState.selectThread(thread) })
                }
            }
        }
    }
}

@Composable
fun ForumPostItem(thread: Channel, forumChannel: Channel, onClick: () -> Unit) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Icon
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .background(MaterialTheme.colorScheme.surfaceVariant, CircleShape),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Filled.Tag,
                    contentDescription = null,
                    modifier = Modifier.size(24.dp),
                    tint = MaterialTheme.colorScheme.primary
                )
            }
            
            Spacer(Modifier.width(12.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = thread.name ?: "unnamed post",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                
                Row(verticalAlignment = Alignment.CenterVertically) {
                    // Applied Tags
                    val appliedTags = thread.applied_tags ?: emptyList()
                    val availableTags = forumChannel.available_tags ?: emptyList()
                    
                    appliedTags.forEach { tagId ->
                        val tag = availableTags.find { it.id == tagId }
                        if (tag != null) {
                            Surface(
                                color = MaterialTheme.colorScheme.secondaryContainer,
                                shape = RoundedCornerShape(4.dp),
                                modifier = Modifier.padding(end = 4.dp)
                            ) {
                                Text(
                                    text = tag.name,
                                    style = MaterialTheme.typography.labelSmall,
                                    modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp),
                                    color = MaterialTheme.colorScheme.onSecondaryContainer
                                )
                            }
                        }
                    }

                    Text(
                        text = "${thread.message_count ?: 0} messages",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
            
            // Last active timestamp / user (placeholder)
            if (thread.last_message_id != null) {
                Column(horizontalAlignment = Alignment.End) {
                    Text(
                        text = "Last message",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Composable
fun ForumSkeleton() {
    Surface(
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(8.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            ShimmerBox(Modifier.size(40.dp), shape = CircleShape)
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                ShimmerBox(Modifier.width(200.dp).height(18.dp), shape = RoundedCornerShape(9.dp))
                Spacer(Modifier.height(8.dp))
                ShimmerBox(Modifier.width(100.dp).height(12.dp), shape = RoundedCornerShape(6.dp))
            }
        }
    }
}
