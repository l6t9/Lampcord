package com.example.lampcord.shared.ui.components.messagebody

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
import com.example.lampcord.shared.model.StickerItem
import com.example.lampcord.shared.ui.components.AsyncImage

@Composable
fun StickersView(stickers: List<StickerItem>) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        stickers.forEach { sticker ->
            Surface(
                color = Color.Transparent,
                shape = RoundedCornerShape(8.dp),
                onClick = { /* TODO: Sticker Info */ }
            ) {
                AsyncImage(
                    model = "https://cdn.discordapp.com/stickers/${sticker.id}.png?size=320",
                    contentDescription = sticker.name,
                    modifier = Modifier.size(160.dp).clip(RoundedCornerShape(8.dp)),
                    contentScale = ContentScale.Fit,
                )
            }
        }
    }
}
