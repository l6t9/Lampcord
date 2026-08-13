package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp

@Composable
expect fun ContextMenu(
    items: List<ContextMenuItem>,
    modifier: Modifier = Modifier,
    shape: androidx.compose.ui.graphics.Shape = RoundedCornerShape(0.dp),
    header: (@Composable () -> Unit)? = null,
    reactions: (@Composable (onDismiss: () -> Unit) -> Unit)? = null,
    content: @Composable () -> Unit
)

data class ContextMenuItem(
    val label: String,
    val icon: ImageVector? = null,
    val color: Color? = null,
    val onClick: () -> Unit
)
