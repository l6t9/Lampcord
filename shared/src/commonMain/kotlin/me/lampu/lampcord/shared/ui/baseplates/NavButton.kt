package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun NavButtonRow(
    buttons: List<NavButtonData>,
    modifier: Modifier = Modifier
) {
    if (buttons.isEmpty()) return

    Surface(
        modifier = modifier
            .height(84.dp)
            .fillMaxWidth(),
        shape = MaterialTheme.shapes.medium,
        tonalElevation = 1.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(6.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            buttons.forEachIndexed { index, button ->
                val shape = when {
                    buttons.size == 1 -> RoundedCornerShape(28.dp)
                    index == 0 -> RoundedCornerShape(topStart = 28.dp, bottomStart = 28.dp, topEnd = 6.dp, bottomEnd = 6.dp)
                    index == buttons.lastIndex -> RoundedCornerShape(topStart = 6.dp, bottomStart = 6.dp, topEnd = 28.dp, bottomEnd = 28.dp)
                    else -> RoundedCornerShape(6.dp)
                }
                
                Surface(
                    onClick = button.onClick,
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight()
                        .semantics { role = Role.Button },
                    shape = shape,
                    color = if (button.selected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.secondaryContainer,
                    contentColor = if (button.selected) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSecondaryContainer,
                ) {
                    Column(
                        modifier = Modifier.fillMaxSize().padding(horizontal = 4.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center
                    ) {
                        Icon(
                            imageVector = button.icon,
                            contentDescription = button.contentDescription,
                            modifier = Modifier.size(24.dp)
                        )
                        if (button.title != null) {
                            Spacer(Modifier.height(4.dp))
                            Text(
                                text = button.title,
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontSize = 10.sp,
                                    letterSpacing = 0.5.sp
                                ),
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

data class NavButtonData(
    val icon: ImageVector,
    val contentDescription: String? = null,
    val selected: Boolean = false,
    val title: String? = null,
    val onClick: () -> Unit
)
