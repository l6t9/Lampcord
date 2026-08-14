package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.AvatarDecorationData

@Composable
fun AvatarWithDecoration(
    avatarUrl: String?,
    decorationData: AvatarDecorationData?,
    size: Dp,
    modifier: Modifier = Modifier,
    status: String? = null
) {
    Box(modifier = modifier.size(size)) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            shape = CircleShape,
            color = MaterialTheme.colorScheme.secondaryContainer
        ) {
            if (avatarUrl != null) {
                AsyncImage(
                    model = avatarUrl,
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = androidx.compose.ui.layout.ContentScale.Crop,
                    filterQuality = androidx.compose.ui.graphics.FilterQuality.High
                )
            }
        }

        if (decorationData != null) {
            val decorationUrl = "https://cdn.discordapp.com/avatar-decoration-presets/${decorationData.asset}.png"
            AsyncImage(
                model = decorationUrl,
                contentDescription = null,
                modifier = Modifier
                    .fillMaxSize()
                    .scale(1.15f),
                filterQuality = androidx.compose.ui.graphics.FilterQuality.High
            )
        }

        if (status != null) {
            StatusIndicator(
                status = status,
                size = size * 0.35f,
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .offset(x = 2.dp, y = 2.dp),
                borderColor = MaterialTheme.colorScheme.surface
            )
        }
    }
}
