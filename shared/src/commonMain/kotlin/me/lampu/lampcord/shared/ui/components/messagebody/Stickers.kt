package me.lampu.lampcord.shared.ui.components.messagebody

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.StickerItem
import me.lampu.lampcord.shared.ui.components.AsyncImage

@Composable
fun StickersView(stickers: List<StickerItem>) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        stickers.forEach { sticker ->
            val stickerUrl = when (sticker.format_type) {
                4 -> "https://cdn.discordapp.com/stickers/${sticker.id}.gif?size=320"
                3 -> "https://cdn.discordapp.com/stickers/${sticker.id}.json" // Lottie, needs special handling for full animation
                else -> "https://cdn.discordapp.com/stickers/${sticker.id}.png?size=320"
            }

            Surface(
                color = Color.Transparent,
                shape = RoundedCornerShape(8.dp),
                onClick = { /* TODO: Sticker Info */ }
            ) {
                AsyncImage(
                    model = stickerUrl,
                    contentDescription = sticker.name,
                    modifier = Modifier.size(160.dp).clip(RoundedCornerShape(8.dp)),
                    contentScale = ContentScale.Fit,
                )
            }
        }
    }
}
