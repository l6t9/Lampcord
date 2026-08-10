package me.lampu.lampcord.shared.ui.components.profiles

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.ui.components.AsyncImage

@Composable
fun ProfileBanner(
    profile: UserProfile,
    theme: ProfileTheme,
    isExpanded: Boolean
) {
    val user = profile.user
    val guildMeta = profile.guild_member_profile
    val userMeta = profile.user_profile
    
    Box(modifier = Modifier.fillMaxWidth().height(if (isExpanded) 160.dp else 105.dp)) {
        val bannerUrl = guildMeta?.banner ?: userMeta?.banner ?: user.banner
        if (bannerUrl != null) {
            AsyncImage(
                model = "https://cdn.discordapp.com/banners/${user.id}/$bannerUrl.png?size=600",
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
                filterQuality = FilterQuality.Medium
            )
        }
    }
}
