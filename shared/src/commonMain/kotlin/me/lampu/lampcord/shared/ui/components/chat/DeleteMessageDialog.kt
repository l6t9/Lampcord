package me.lampu.lampcord.shared.ui.components.chat

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AlertDialogDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.ui.icons.Icons

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun DeleteMessageDialog(
    onDismiss: () -> Unit,
    onConfirm: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        icon = {
            Icon(
                Icons.Filled.Delete,
                contentDescription = null,
                modifier = Modifier.size(32.dp),
                tint = MaterialTheme.colorScheme.error
            )
        },
        title = {
            Text(
                text = "Delete Message",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        },
        text = {
            Text(
                text = "Are you sure you want to delete this message? This cannot be undone.",
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.fillMaxWidth()
            )
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
                            onClick = onConfirm,
                            shapes = ButtonDefaults.shapes(
                                shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape,
                            ),
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.error,
                                contentColor = MaterialTheme.colorScheme.onError
                            ),
                            modifier = Modifier.weight(1f).fillMaxHeight()
                        ) {
                            Text("Delete")
                        }
                    },
                    menuContent = { menuState ->
                        DropdownMenuItem(
                            text = { Text("Delete") },
                            onClick = { 
                                onConfirm()
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
