package me.lampu.lampcord.shared.ui.baseplates

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp

@Composable
fun RowScope.NavButton(icon: ImageVector, onClick: () -> Unit) {
    Surface(
        modifier = Modifier.weight(1f).fillMaxHeight(),
        shape = MaterialTheme.shapes.medium,
        tonalElevation = 1.dp
    ) {
        Box(
            modifier = Modifier.fillMaxSize().clickable(onClick = onClick),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(24.dp).alpha(0.85f)
            )
        }
    }
}
