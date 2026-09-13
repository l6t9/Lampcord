package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import kotlinx.serialization.json.*
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.state.ProfileStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import org.koin.compose.koinInject

@Composable
fun ProfileEffectView(
    skuId: String,
    modifier: Modifier = Modifier,
    profileStore: ProfileStore = koinInject()
) {
    val collectible = profileStore.getCollectible(skuId) ?: return
    val items = collectible["items"]?.jsonArray?.firstOrNull()?.jsonObject ?: return
    val effects = items["effects"]?.jsonArray
    
    Box(modifier = modifier.fillMaxSize()) {
        if (effects != null && !Settings.shared.reduceMotion) {
            effects.forEach { effectElement ->
                val effect = effectElement.jsonObject
                val src = effect["animatedSrc"]?.jsonPrimitive?.content ?: effect["src"]?.jsonPrimitive?.content ?: return@forEach
                
                AsyncImage(
                    model = formatCdnUrl(src),
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    showPlaceholder = false
                )
            }
        } else {
            val staticSrc = items["staticFrameSrc"]?.jsonPrimitive?.content
                ?: items["reducedMotionSrc"]?.jsonPrimitive?.content
                ?: effects?.firstOrNull()?.jsonObject?.get("src")?.jsonPrimitive?.content ?: return
                
            AsyncImage(
                model = formatCdnUrl(staticSrc),
                contentDescription = null,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
                showPlaceholder = false
            )
        }
    }
}

private fun formatCdnUrl(src: String): String {
    if (src.startsWith("http")) return src
    if (src.startsWith("//")) return "https:$src"
    if (src.startsWith("/")) return "https://cdn.discordapp.com$src"
    return "https://cdn.discordapp.com/$src"
}
