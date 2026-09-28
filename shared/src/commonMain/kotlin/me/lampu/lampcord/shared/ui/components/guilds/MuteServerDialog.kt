package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import me.lampu.lampcord.shared.ui.kit.handCursor

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MuteServerDialog(
    guildName: String,
    onDismiss: () -> Unit,
    onConfirm: (Duration?) -> Unit
) {
    var selectedOption by remember { mutableStateOf<Duration?>(Duration.INFINITE) }
    
    val options = listOf(
        15.minutes to "For 15 minutes",
        1.hours to "For 1 hour",
        8.hours to "For 8 hours",
        24.hours to "For 24 hours",
        Duration.INFINITE to "Until I turn it back on"
    )

    AlertDialog(
        onDismissRequest = onDismiss,
        icon = {
            Icon(
                imageVector = Icons.AutoMirrored.Filled.VolumeOff,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = MaterialTheme.colorScheme.primary
            )
        },
        title = {
            Text(
                text = "Mute '$guildName'",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        },
        text = {
            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "Muting this server will suppress unread indicators and notifications according to your duration choice.",
                    style = MaterialTheme.typography.bodyMedium,
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(bottom = 16.dp)
                )

                Surface(
                    shape = MaterialTheme.shapes.medium,
                    color = MaterialTheme.colorScheme.surfaceContainerLow,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(vertical = 8.dp)) {
                        options.forEach { (duration, label) ->
                            Row(
                                Modifier
                                    .fillMaxWidth()
                                    .selectable(
                                        selected = (selectedOption == duration),
                                        onClick = { selectedOption = duration }
                                    )
                                    .padding(horizontal = 16.dp, vertical = 12.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                RadioButton(
                                    selected = (selectedOption == duration),
                                    onClick = null
                                )
                                Spacer(Modifier.width(16.dp))
                                Text(
                                    text = label,
                                    style = MaterialTheme.typography.bodyLarge
                                )
                            }
                        }
                    }
                }
            }
        },
        confirmButton = {
            ButtonGroup(
                overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
                modifier = Modifier.fillMaxWidth().height(64.dp)
            ) {
                customItem(
                    buttonGroupContent = {
                        Button(
                            onClick = onDismiss,
                            shapes = ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape,
                            ),
                            colors = ButtonDefaults.filledTonalButtonColors(),
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        ) {
                            Text("Cancel")
                        }
                    },
                    menuContent = { menuState ->
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Cancel") },
                            onClick = { 
                                onDismiss()
                                menuState.dismiss() 
                            }
                        )
                    }
                )
                customItem(
                    buttonGroupContent = {
                        Button(
                            onClick = { onConfirm(selectedOption) },
                            shapes = ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape,
                            ),
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        ) {
                            Text("Mute")
                        }
                    },
                    menuContent = { menuState ->
                        DropdownMenuItem(
                            modifier = Modifier.handCursor(),
                            text = { Text("Mute") },
                            onClick = { 
                                onConfirm(selectedOption)
                                menuState.dismiss() 
                            }
                        )
                    }
                )
            }
        },
        shape = MaterialTheme.shapes.extraLarge,
        tonalElevation = AlertDialogDefaults.TonalElevation
    )
}
