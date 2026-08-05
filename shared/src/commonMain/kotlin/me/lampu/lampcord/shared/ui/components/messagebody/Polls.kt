package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.ui.icons.Icons

@Composable
fun PollView(poll: Poll) {
    Surface(
        modifier = Modifier.padding(vertical = 4.dp).widthIn(max = 425.dp).fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(12.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Filled.BarChart, null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.primary)
                Spacer(Modifier.width(8.dp))
                Text(text = "Poll", style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.primary)
            }
            Spacer(Modifier.height(8.dp))
            Text(text = poll.question.text ?: "Untitled Poll", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            Spacer(Modifier.height(12.dp))
            val totalVotes = poll.results?.answer_counts?.sumOf { it.count } ?: 0
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                poll.answers.forEach { answer ->
                    val result = poll.results?.answer_counts?.find { it.id == answer.answer_id }
                    val count = result?.count ?: 0
                    val percentage = if (totalVotes > 0) count.toFloat() / totalVotes.toFloat() else 0f
                    Column {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                            Text(text = answer.poll_media.text ?: "", style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
                            if (totalVotes > 0) {
                                Text(text = "$count (${(percentage * 100).toInt()}%)", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                        Spacer(Modifier.height(4.dp))
                        LinearProgressIndicator(
                            progress = { percentage },
                            modifier = Modifier.fillMaxWidth().height(8.dp).clip(CircleShape),
                            color = if (result?.me_voted == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.primary.copy(alpha = 0.4f),
                            trackColor = MaterialTheme.colorScheme.surfaceVariant
                        )
                    }
                }
            }
            if (totalVotes > 0) {
                Spacer(Modifier.height(12.dp))
                Text(text = "$totalVotes votes", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}
