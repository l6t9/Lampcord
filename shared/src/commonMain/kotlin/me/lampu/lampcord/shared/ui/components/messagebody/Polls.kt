package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlin.time.Clock
import kotlin.time.Instant
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.api.MessageApi
import me.lampu.lampcord.shared.model.Message
import me.lampu.lampcord.shared.model.Poll
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.kit.clickableCursor
import org.koin.compose.koinInject

@Composable
fun PollView(
    poll: Poll,
    message: Message? = null,
    messageApi: MessageApi = koinInject(),
    userStore: UserStore = koinInject()
) {
    var currentPoll by remember(poll) { mutableStateOf(poll) }
    val scope = rememberCoroutineScope()

    val isFinalized = currentPoll.results?.is_finalized == true
    val isExpired = remember(currentPoll.expiry, isFinalized) {
        if (isFinalized) return@remember true
        val expiryIso = currentPoll.expiry ?: return@remember false
        try {
            Instant.parse(expiryIso) <= Clock.System.now()
        } catch (e: Exception) {
            false
        }
    }
    val isActive = !isExpired && !isFinalized

    // Determine if user has already voted
    val myVotedAnswerIds = remember(currentPoll.results) {
        currentPoll.results?.answer_counts?.filter { it.me_voted }?.map { it.id }?.toSet() ?: emptySet()
    }
    val hasVoted = myVotedAnswerIds.isNotEmpty()

    // Show results mode state (default to showing results if user voted or poll expired)
    var showResults by remember(hasVoted, isExpired) { mutableStateOf(hasVoted || isExpired) }

    // Selected answers state for active voting
    var selectedAnswerIds by remember(myVotedAnswerIds) { mutableStateOf(myVotedAnswerIds) }
    var isSubmitting by remember { mutableStateOf(false) }

    val totalVotes = currentPoll.results?.answer_counts?.sumOf { it.count } ?: 0
    val currentUser = userStore.currentUser.value
    val isAuthor = message?.author?.id == currentUser?.id

    val expiryText = remember(currentPoll.expiry, isExpired, isFinalized) {
        formatPollExpiry(currentPoll.expiry, isExpired || isFinalized)
    }

    Surface(
        modifier = Modifier
            .padding(vertical = 4.dp)
            .widthIn(max = 450.dp)
            .fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(12.dp),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            // Header: Poll Icon + Label + Multiselect hint + Expiry Duration
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        imageVector = Icons.Filled.BarChart,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                        tint = MaterialTheme.colorScheme.primary
                    )
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = if (currentPoll.allow_multiselect) "Poll • Select one or more" else "Poll • Select 1",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.primary
                    )
                }

                if (expiryText.isNotBlank()) {
                    Text(
                        text = expiryText,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            Spacer(Modifier.height(8.dp))

            // Question
            Text(
                text = currentPoll.question.text ?: "Untitled Poll",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface
            )

            Spacer(Modifier.height(12.dp))

            // Options List
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                currentPoll.answers.forEach { answer ->
                    val answerResult = currentPoll.results?.answer_counts?.find { it.id == answer.answer_id }
                    val count = answerResult?.count ?: 0
                    val meVoted = myVotedAnswerIds.contains(answer.answer_id)
                    val isSelected = selectedAnswerIds.contains(answer.answer_id)
                    val percentage = if (totalVotes > 0) count.toFloat() / totalVotes.toFloat() else 0f

                    if (showResults) {
                        // Results View Mode (Progress Bar + Percentage)
                        Surface(
                            shape = RoundedCornerShape(8.dp),
                            color = MaterialTheme.colorScheme.surfaceContainer,
                            border = if (meVoted) BorderStroke(1.5.dp, MaterialTheme.colorScheme.primary) else null,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Column(modifier = Modifier.padding(10.dp)) {
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        modifier = Modifier.weight(1f)
                                    ) {
                                        if (meVoted) {
                                            Icon(
                                                imageVector = Icons.Filled.Check,
                                                contentDescription = "Voted",
                                                modifier = Modifier.size(16.dp),
                                                tint = MaterialTheme.colorScheme.primary
                                            )
                                            Spacer(Modifier.width(6.dp))
                                        }
                                        Text(
                                            text = answer.poll_media.text ?: "",
                                            style = MaterialTheme.typography.bodyMedium,
                                            fontWeight = if (meVoted) FontWeight.Bold else FontWeight.Normal,
                                            color = MaterialTheme.colorScheme.onSurface
                                        )
                                    }

                                    Text(
                                        text = "$count (${(percentage * 100).toInt()}%)",
                                        style = MaterialTheme.typography.labelSmall,
                                        fontWeight = if (meVoted) FontWeight.Bold else FontWeight.Normal,
                                        color = if (meVoted) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }

                                Spacer(Modifier.height(6.dp))

                                LinearProgressIndicator(
                                    progress = { percentage },
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(8.dp)
                                        .clip(CircleShape),
                                    color = if (meVoted) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.primary.copy(alpha = 0.4f),
                                    trackColor = MaterialTheme.colorScheme.surfaceVariant
                                )
                            }
                        }
                    } else {
                        // Voting Mode (Selectable Option Card)
                        Surface(
                            shape = RoundedCornerShape(8.dp),
                            color = if (isSelected) MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f) else MaterialTheme.colorScheme.surfaceContainer,
                            border = BorderStroke(
                                1.dp,
                                if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant
                            ),
                            modifier = Modifier
                                .fillMaxWidth()
                                .then(
                                    if (isActive && !isSubmitting) {
                                        Modifier.clickableCursor {
                                            if (currentPoll.allow_multiselect) {
                                                selectedAnswerIds = if (isSelected) {
                                                    selectedAnswerIds - answer.answer_id
                                                } else {
                                                    selectedAnswerIds + answer.answer_id
                                                }
                                            } else {
                                                selectedAnswerIds = setOf(answer.answer_id)
                                            }
                                        }
                                    } else Modifier
                                )
                        ) {
                            Row(
                                modifier = Modifier.padding(12.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                if (currentPoll.allow_multiselect) {
                                    Checkbox(
                                        checked = isSelected,
                                        onCheckedChange = { checked ->
                                            if (isActive && !isSubmitting) {
                                                selectedAnswerIds = if (checked) {
                                                    selectedAnswerIds + answer.answer_id
                                                } else {
                                                    selectedAnswerIds - answer.answer_id
                                                }
                                            }
                                        },
                                        enabled = isActive && !isSubmitting
                                    )
                                } else {
                                    RadioButton(
                                        selected = isSelected,
                                        onClick = {
                                            if (isActive && !isSubmitting) {
                                                selectedAnswerIds = setOf(answer.answer_id)
                                            }
                                        },
                                        enabled = isActive && !isSubmitting
                                    )
                                }

                                Spacer(Modifier.width(8.dp))

                                Text(
                                    text = answer.poll_media.text ?: "",
                                    style = MaterialTheme.typography.bodyMedium,
                                    fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                                    color = MaterialTheme.colorScheme.onSurface,
                                    modifier = Modifier.weight(1f)
                                )
                            }
                        }
                    }
                }
            }

            Spacer(Modifier.height(12.dp))

            // Footer: Vote button / Toggle Results / End Poll
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    if (isActive && !showResults) {
                        Button(
                            onClick = {
                                if (message != null && selectedAnswerIds != myVotedAnswerIds && !isSubmitting) {
                                    isSubmitting = true
                                    scope.launch {
                                        val channelId = message.channel_id
                                        val messageId = message.id
                                        
                                        val toRemove = myVotedAnswerIds - selectedAnswerIds
                                        val toAdd = selectedAnswerIds - myVotedAnswerIds
                                        
                                        var updatedMsg: Message? = null
                                        for (ansId in toRemove) {
                                            updatedMsg = messageApi.unvotePollAnswer(channelId, messageId, ansId) ?: updatedMsg
                                        }
                                        for (ansId in toAdd) {
                                            updatedMsg = messageApi.votePollAnswer(channelId, messageId, ansId) ?: updatedMsg
                                        }

                                        if (updatedMsg != null) {
                                            updatedMsg.poll?.let { currentPoll = it }
                                        }
                                        isSubmitting = false
                                        showResults = true
                                    }
                                }
                            },
                            enabled = isActive && selectedAnswerIds.isNotEmpty() && selectedAnswerIds != myVotedAnswerIds && !isSubmitting,
                            contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp),
                            modifier = Modifier.height(36.dp)
                        ) {
                            Text("Vote", style = MaterialTheme.typography.labelLarge)
                        }
                    }

                    if (isActive) {
                        TextButton(
                            onClick = { showResults = !showResults },
                            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 4.dp)
                        ) {
                            Text(
                                text = if (showResults) "Show Options" else "Show Results",
                                style = MaterialTheme.typography.labelMedium
                            )
                        }
                    }
                }

                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Text(
                        text = if (totalVotes == 1) "1 vote" else "$totalVotes votes",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )

                    if (isActive && isAuthor && message != null) {
                        TextButton(
                            onClick = {
                                scope.launch {
                                    val updatedMsg = messageApi.expirePoll(message.channel_id, message.id)
                                    if (updatedMsg != null) {
                                        updatedMsg.poll?.let { currentPoll = it }
                                    }
                                }
                            },
                            contentPadding = PaddingValues(horizontal = 6.dp, vertical = 2.dp)
                        ) {
                            Text(
                                text = "End Poll",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.error
                            )
                        }
                    }
                }
            }
        }
    }
}

private fun formatPollExpiry(expiryIso: String?, isClosed: Boolean): String {
    if (isClosed) return "Final results"
    if (expiryIso == null) return ""
    return try {
        val expiryInstant = Instant.parse(expiryIso)
        val now = Clock.System.now()
        if (now >= expiryInstant) {
            "Final results"
        } else {
            val remainingSeconds = (expiryInstant - now).inWholeSeconds
            val text = when {
                remainingSeconds < 60 -> "$remainingSeconds sec"
                remainingSeconds < 3600 -> "${remainingSeconds / 60}m"
                remainingSeconds < 86400 -> "${remainingSeconds / 3600}h"
                else -> "${remainingSeconds / 86400}d"
            }
            "Ends in $text"
        }
    } catch (e: Exception) {
        "Final results"
    }
}
