package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
fun SettingsCategoryItem(
    label: String,
    icon: ImageVector,
    isSelected: Boolean,
    color: Color = Color.Unspecified,
    isFirst: Boolean = false,
    isLast: Boolean = false,
    onClick: () -> Unit
) {
    val cornerRadius = 24.dp
    val reducedRadius = 4.dp
    
    val shape = RoundedCornerShape(
        topStart = if (isFirst) cornerRadius else reducedRadius,
        topEnd = if (isFirst) cornerRadius else reducedRadius,
        bottomStart = if (isLast) cornerRadius else reducedRadius,
        bottomEnd = if (isLast) cornerRadius else reducedRadius
    )

    Surface(
        modifier = Modifier.fillMaxWidth().height(56.dp),
        onClick = onClick,
        color = if (isSelected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainer,
        shape = shape
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = if (color != Color.Unspecified) color else if (isSelected) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.width(16.dp))
            Text(
                text = label,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                color = if (color != Color.Unspecified) color else if (isSelected) MaterialTheme.colorScheme.onSecondaryContainer else MaterialTheme.colorScheme.onSurface
            )
        }
    }
}
