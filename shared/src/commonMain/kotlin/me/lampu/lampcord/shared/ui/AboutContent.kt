package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.AsyncImage

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun AboutContent(
    version: String,
    onOpenUrl: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        // App Header Card
        ElevatedCard(
            shape = RoundedCornerShape(32.dp),
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.elevatedCardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth().padding(24.dp),
            ) {
                Box(
                    modifier = Modifier
                        .size(64.dp)
                        .background(MaterialTheme.colorScheme.primary, RoundedCornerShape(16.dp)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        Icons.Brand.Discord,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimary,
                        modifier = Modifier.size(40.dp)
                    )
                }
                Spacer(Modifier.width(20.dp))
                Column {
                    Text(
                        text = "Materialcord",
                        style = MaterialTheme.typography.headlineLarge,
                        fontWeight = FontWeight.Black,
                        color = MaterialTheme.colorScheme.onSurface,
                        letterSpacing = (-0.5).sp,
                    )
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        InfoBadge(version, MaterialTheme.colorScheme.primary.copy(alpha = 0.1f), MaterialTheme.colorScheme.primary)
                    }
                }
            }
        }

        Spacer(Modifier.height(32.dp))

        // Contributors Section
        Column(Modifier.fillMaxWidth()) {
            Text(
                text = "Contributors",
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Medium),
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(top = 8.dp, bottom = 12.dp),
            )
            
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onOpenUrl("https://lamp.delivery") },
                shape = RoundedCornerShape(20.dp),
                color = MaterialTheme.colorScheme.surfaceContainer,
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    val avatarShape = MaterialShapes.Cookie9Sided.toShape()
                    
                    Surface(
                        modifier = Modifier.size(48.dp),
                        shape = avatarShape,
                        color = MaterialTheme.colorScheme.surfaceContainerHighest,
                        tonalElevation = 4.dp,
                    ) {
                        AsyncImage(
                            model = "https://github.com/l6t9.png",
                            contentDescription = "l6t9",
                            modifier = Modifier.fillMaxSize(),
                            contentScale = ContentScale.Crop,
                            shape = avatarShape
                        )
                    }
                    
                    Spacer(Modifier.width(16.dp))
                    Column(Modifier.weight(1f)) {
                        Text(text = "Lamp", fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                        Text(
                            text = "Main Developer",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    IconButton(
                        onClick = { onOpenUrl("https://github.com/sponsors/l6t9") },
                        modifier = Modifier.size(48.dp),
                    ) {
                        Surface(
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f),
                            modifier = Modifier.size(36.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    Icons.Filled.Favorite,
                                    contentDescription = "Sponsor",
                                    modifier = Modifier.size(20.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(32.dp))

        // Links Section
        val links = listOf(
            Triple("Discord", "https://discord.gg/materialcord", Icons.Brand.Discord),
            Triple("GitHub Repository", "https://github.com/l6t9/Materialcord", Icons.Brand.Github),
            Triple("License", "https://github.com/l6t9/Materialcord/blob/main/LICENSE", Icons.Filled.Info)
        )
        
        AboutSection(
            title = "Community & Info",
            itemCount = links.size,
            itemContent = { index ->
                val (name, _, icon) = links[index]
                Row(
                    modifier = Modifier.fillMaxWidth().padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(24.dp),
                    )
                    Spacer(Modifier.width(16.dp))
                    Text(text = name, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                }
            },
            onClick = { index -> onOpenUrl(links[index].second) }
        )
        
        Spacer(Modifier.height(32.dp))
    }
}

@Composable
private fun InfoBadge(
    label: String,
    containerColor: androidx.compose.ui.graphics.Color,
    contentColor: androidx.compose.ui.graphics.Color,
) {
    Surface(shape = RoundedCornerShape(8.dp), color = containerColor) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.Bold,
            color = contentColor,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
        )
    }
}

@Composable
private fun AboutSection(
    title: String,
    itemCount: Int,
    itemContent: @Composable (Int) -> Unit,
    onClick: (Int) -> Unit,
) {
    Column(Modifier.fillMaxWidth()) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Medium),
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(top = 8.dp, bottom = 12.dp),
        )
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            repeat(itemCount) { index ->
                val shape =
                    RoundedCornerShape(
                        topStart = if (index == 0) 20.dp else 5.dp,
                        topEnd = if (index == 0) 20.dp else 5.dp,
                        bottomStart = if (index == itemCount - 1) 20.dp else 5.dp,
                        bottomEnd = if (index == itemCount - 1) 20.dp else 5.dp,
                    )
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { onClick(index) },
                    shape = shape,
                    color = MaterialTheme.colorScheme.surfaceContainer,
                ) {
                    itemContent(index)
                }
            }
        }
    }
}
