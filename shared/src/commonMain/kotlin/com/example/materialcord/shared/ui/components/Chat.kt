package com.example.materialcord.shared.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.Attachment
import com.example.materialcord.shared.model.Message
import com.example.materialcord.shared.state.ChatState
import com.example.materialcord.shared.ui.icons.MaterialcordIcons

@Composable
fun ChatArea(
    modifier: Modifier = Modifier,
    chatState: ChatState
) {
    LazyColumn(
        modifier = modifier.fillMaxSize().padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        reverseLayout = true
    ) {
        items(chatState.messages) { message ->
            MessageItem(message, chatState)
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MessageItem(message: Message, chatState: ChatState) {
    var isHovered by remember { mutableStateOf(false) }
    
    val contextMenuItems = listOf(
        ContextMenuItem("Copy Text", MaterialcordIcons.Filled.ContentCopy) { /* TODO */ },
        ContextMenuItem("Edit Message", MaterialcordIcons.Filled.Edit) { /* TODO */ },
        ContextMenuItem("Pin Message", MaterialcordIcons.Filled.PushPin) { /* TODO */ },
        ContextMenuItem("Delete Message", MaterialcordIcons.Filled.Delete) { /* TODO */ }
    )

    Box(
        modifier = Modifier
            .fillMaxWidth()
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
            .background(if (isHovered) MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.2f) else Color.Transparent)
            .padding(vertical = 4.dp, horizontal = 16.dp)
    ) {
        MaterialcordContextMenu(
            items = contextMenuItems,
            modifier = Modifier.fillMaxWidth()
        ) {
            Row(modifier = Modifier.fillMaxWidth()) {
                val avatarUrl = message.member?.avatar?.let {
                    "https://cdn.discordapp.com/guilds/${message.guild_id ?: chatState.selectedGuild?.id}/users/${message.author.id}/avatars/$it.png"
                } ?: message.author.avatar?.let {
                    "https://cdn.discordapp.com/avatars/${message.author.id}/$it.png"
                }

                Surface(
                    modifier = Modifier.size(40.dp),
                    onClick = { chatState.showProfile(message.author.id) },
                    shape = MaterialTheme.shapes.extraLarge,
                    color = Color.Transparent
                ) {
                    if (avatarUrl != null) {
                        AsyncImage(
                            model = avatarUrl,
                            contentDescription = "Avatar",
                            modifier = Modifier.fillMaxSize().clip(MaterialTheme.shapes.extraLarge)
                        )
                    } else {
                        Surface(
                            modifier = Modifier.fillMaxSize(),
                            shape = MaterialTheme.shapes.extraLarge,
                            color = MaterialTheme.colorScheme.primaryContainer
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text(
                                    message.author.username.take(1).uppercase(),
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                            }
                        }
                    }
                }
                
                val roleColor = remember(message.member, chatState.selectedGuild) {
                    val guild = chatState.selectedGuild ?: return@remember Color.Unspecified
                    val member = message.member ?: return@remember Color.Unspecified
                    val memberRoles = member.roles.mapNotNull { roleId -> guild.roles.find { it.id == roleId } }
                    val highestRole = memberRoles.maxByOrNull { it.position }
                    if (highestRole != null && highestRole.color != 0) Color(highestRole.color or 0xFF000000.toInt()) else Color.Unspecified
                }

                Spacer(modifier = Modifier.width(12.dp))
                
                Column {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = message.member?.nick ?: message.author.global_name ?: message.author.username,
                            style = MaterialTheme.typography.titleSmall,
                            color = if (roleColor != Color.Unspecified) roleColor else MaterialTheme.colorScheme.primary,
                            modifier = Modifier.clickable { chatState.showProfile(message.author.id) }
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = message.timestamp.take(10), // Simplistic timestamp
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                    Text(
                        text = message.content,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    if (message.attachments.isNotEmpty()) {
                        Spacer(modifier = Modifier.height(8.dp))
                        MessageAttachments(message.attachments)
                    }
                }
            }
        }

        // Hover Toolbar
        if (isHovered) {
            Surface(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(end = 16.dp, top = 0.dp)
                    .offset(y = (-12).dp),
                shape = RoundedCornerShape(4.dp),
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                border = androidx.compose.foundation.BorderStroke(0.5.dp, MaterialTheme.colorScheme.outlineVariant),
                shadowElevation = 2.dp
            ) {
                Row(modifier = Modifier.padding(horizontal = 4.dp)) {
                    IconButton(onClick = { /* TODO */ }, modifier = Modifier.size(32.dp)) {
                        Icon(MaterialcordIcons.Filled.Add, null, modifier = Modifier.size(18.dp))
                    }
                    IconButton(onClick = { /* TODO */ }, modifier = Modifier.size(32.dp)) {
                        Icon(MaterialcordIcons.Filled.Edit, null, modifier = Modifier.size(18.dp))
                    }
                    IconButton(onClick = { /* TODO */ }, modifier = Modifier.size(32.dp)) {
                        Icon(MaterialcordIcons.Filled.MoreHoriz, null, modifier = Modifier.size(18.dp))
                    }
                }
            }
        }
    }
}

@Composable
fun MessageAttachments(attachments: List<Attachment>) {
    val images = attachments.filter { it.content_type?.startsWith("image/") == true }
    
    if (images.isNotEmpty()) {
        MessageMosaic(images)
    }
    
    // Non-image attachments could be listed here
}

@Composable
fun MessageMosaic(images: List<Attachment>) {
    val spacing = 4.dp
    val maxWidth = 500.dp
    
    Box(modifier = Modifier.widthIn(max = maxWidth).clip(RoundedCornerShape(8.dp))) {
        when (images.size) {
            1 -> {
                val image = images[0]
                val aspectRatio = if (image.width != null && image.height != null) {
                    image.width.toFloat() / image.height.toFloat()
                } else 1f
                
                AsyncImage(
                    model = image.proxy_url,
                    contentDescription = image.filename,
                    modifier = Modifier
                        .fillMaxWidth()
                        .aspectRatio(aspectRatio.coerceIn(0.5f, 2f))
                        .clip(RoundedCornerShape(8.dp))
                )
            }
            2 -> {
                Row(modifier = Modifier.height(200.dp), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                    images.forEach { image ->
                        AsyncImage(
                            model = image.proxy_url,
                            contentDescription = image.filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                    }
                }
            }
            3 -> {
                Row(modifier = Modifier.height(300.dp), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                    AsyncImage(
                        model = images[0].proxy_url,
                        contentDescription = images[0].filename,
                        contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                        modifier = Modifier.weight(1f).fillMaxHeight()
                    )
                    Column(modifier = Modifier.weight(1f).fillMaxHeight(), verticalArrangement = Arrangement.spacedBy(spacing)) {
                        AsyncImage(
                            model = images[1].proxy_url,
                            contentDescription = images[1].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxWidth()
                        )
                        AsyncImage(
                            model = images[2].proxy_url,
                            contentDescription = images[2].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxWidth()
                        )
                    }
                }
            }
            4 -> {
                Column(modifier = Modifier.height(300.dp), verticalArrangement = Arrangement.spacedBy(spacing)) {
                    Row(modifier = Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                        AsyncImage(
                            model = images[0].proxy_url,
                            contentDescription = images[0].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                        AsyncImage(
                            model = images[1].proxy_url,
                            contentDescription = images[1].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                    }
                    Row(modifier = Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                        AsyncImage(
                            model = images[2].proxy_url,
                            contentDescription = images[2].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                        AsyncImage(
                            model = images[3].proxy_url,
                            contentDescription = images[3].filename,
                            contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        )
                    }
                }
            }
            else -> {
                // Simplified grid for more than 4
                val rows = (images.size + 1) / 2
                Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                    for (i in 0 until rows) {
                        Row(modifier = Modifier.height(150.dp), horizontalArrangement = Arrangement.spacedBy(spacing)) {
                            val first = i * 2
                            if (first < images.size) {
                                AsyncImage(
                                    model = images[first].proxy_url,
                                    contentDescription = images[first].filename,
                                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                                    modifier = Modifier.weight(1f).fillMaxHeight()
                                )
                            }
                            val second = i * 2 + 1
                            if (second < images.size) {
                                AsyncImage(
                                    model = images[second].proxy_url,
                                    contentDescription = images[second].filename,
                                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                                    modifier = Modifier.weight(1f).fillMaxHeight()
                                )
                            } else if (first < images.size) {
                                Spacer(modifier = Modifier.weight(1f))
                            }
                        }
                    }
                }
            }
        }
    }
}
