package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.model.ApplicationCommand
import me.lampu.lampcord.shared.state.CommandStore
import org.koin.compose.koinInject

@Composable
fun CommandPicker(
    query: String,
    onCommandSelected: (ApplicationCommand) -> Unit,
    commandStore: CommandStore = koinInject()
) {
    val filteredCommands = commandStore.availableCommands.filter {
        it.name.contains(query, ignoreCase = true)
    }.take(15)

    if (filteredCommands.isEmpty()) return

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(max = 200.dp),
        shape = RoundedCornerShape(16.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        border = androidx.compose.foundation.BorderStroke(
            1.dp,
            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.45f)
        ),
        tonalElevation = 2.dp
    ) {
        Column {
            Text(
                text = if (query.isBlank()) {
                    "Commands"
                } else {
                    "Commands matching \"$query\""
                },
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f))
            LazyColumn(contentPadding = PaddingValues(vertical = 2.dp)) {
                items(filteredCommands, key = { it.id }) { command ->
                    val app = commandStore.availableApplications.find { it.id == command.application_id }
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(48.dp)
                            .clickable(interactionSource = null, indication = null) {
                                onCommandSelected(command)
                            }
                            .padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val iconUrl = app?.icon?.let {
                            "https://cdn.discordapp.com/app-icons/${app.id}/$it.png?size=64"
                        }
                        if (iconUrl != null) {
                            AsyncImage(
                                model = iconUrl,
                                contentDescription = null,
                                modifier = Modifier.size(32.dp).clip(RoundedCornerShape(8.dp)),
                                contentScale = ContentScale.Crop
                            )
                        } else {
                            Box(
                                modifier = Modifier
                                    .size(32.dp)
                                    .background(
                                        MaterialTheme.colorScheme.primaryContainer,
                                        RoundedCornerShape(8.dp)
                                    ),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = "/",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer
                                )
                            }
                        }
                        Spacer(Modifier.width(12.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "/${command.name}",
                                style = MaterialTheme.typography.bodyLarge.copy(fontSize = 16.sp),
                                fontWeight = FontWeight.SemiBold,
                                color = MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            if (command.description != null) {
                                Text(
                                    text = command.description,
                                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 14.sp),
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            }
                        }
                        if (app != null) {
                            Spacer(Modifier.width(12.dp))
                            Text(
                                text = app.name,
                                style = MaterialTheme.typography.bodySmall.copy(fontSize = 14.sp),
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.75f),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }
                }
            }
        }
    }
}
