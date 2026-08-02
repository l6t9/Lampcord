package com.example.materialcord.shared.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.example.materialcord.shared.model.UserProfile
import com.example.materialcord.shared.state.ChatState
import kotlin.math.pow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UserProfileDialog(
    profile: UserProfile,
    chatState: ChatState,
    onDismiss: () -> Unit
) {
    val user = profile.user
    val meta = profile.guild_member_profile ?: profile.user_profile
    
    val themeColors = remember(meta) {
        val colors = meta?.theme_colors
        if (colors != null && colors.isNotEmpty()) {
            if (colors.size >= 2) colors else listOf(colors[0], colors[0])
        } else {
            val accent = meta?.accent_color ?: user.accent_color
            if (accent != null) {
                listOf(accent, accent)
            } else {
                null
            }
        }
    }

    val primaryColor = remember(themeColors) { themeColors?.get(0) ?: 0xFF5865F2.toInt() }
    val cardColor = remember(primaryColor) { Color(ModernProfileColors.softenColor(primaryColor)) }

    BasicAlertDialog(
        onDismissRequest = onDismiss,
        properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(400.dp).padding(16.dp)
    ) {
        Surface(
            modifier = Modifier.fillMaxWidth().wrapContentHeight(),
            shape = RoundedCornerShape(24.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 0.dp
        ) {
            Column {
                // Profile Header (Banner & Avatar)
                Box(modifier = Modifier.fillMaxWidth().height(160.dp)) {
                    // Banner
                    val bannerUrl = meta?.banner ?: user.banner
                    if (bannerUrl != null) {
                        AsyncImage(
                            model = "https://cdn.discordapp.com/banners/${user.id}/$bannerUrl.png?size=600",
                            contentDescription = null,
                            contentScale = ContentScale.Crop,
                            modifier = Modifier.fillMaxWidth().height(100.dp)
                        )
                    } else if (themeColors != null) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(100.dp)
                                .background(
                                    Brush.verticalGradient(
                                        colors = listOf(
                                            Color(themeColors[0] or 0xFF000000.toInt()), 
                                            Color(themeColors[1] or 0xFF000000.toInt())
                                        )
                                    )
                                )
                        )
                    } else {
                        Box(modifier = Modifier.fillMaxWidth().height(100.dp).background(MaterialTheme.colorScheme.primary))
                    }

                    // Avatar
                    val avatarUrl = user.avatar?.let { "https://cdn.discordapp.com/avatars/${user.id}/$it.png?size=160" }
                    Surface(
                        modifier = Modifier.size(90.dp).align(Alignment.BottomStart).padding(start = 16.dp, bottom = 4.dp),
                        shape = CircleShape,
                        border = androidx.compose.foundation.BorderStroke(4.dp, MaterialTheme.colorScheme.surface),
                        color = MaterialTheme.colorScheme.surfaceContainer
                    ) {
                        if (avatarUrl != null) {
                            AsyncImage(model = avatarUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                        } else {
                            Box(contentAlignment = Alignment.Center) {
                                Text(user.username.take(1).uppercase(), style = MaterialTheme.typography.headlineLarge)
                            }
                        }
                    }
                }

                Surface(
                    modifier = Modifier.fillMaxWidth().padding(16.dp),
                    color = cardColor,
                    shape = RoundedCornerShape(16.dp)
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            text = user.global_name ?: user.username,
                            style = MaterialTheme.typography.headlineSmall,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = user.username,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f)
                        )
                        
                        if (meta?.pronouns?.isNotBlank() == true) {
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(text = meta.pronouns, style = MaterialTheme.typography.labelMedium)
                        }

                        HorizontalDivider(
                            modifier = Modifier.padding(vertical = 12.dp), 
                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.1f)
                        )

                        val bio = meta?.bio ?: user.bio
                        if (bio?.isNotBlank() == true) {
                            Text(text = "ABOUT ME", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold)
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(text = bio, style = MaterialTheme.typography.bodyMedium)
                            Spacer(modifier = Modifier.height(12.dp))
                        }

                        if (profile.guild_member?.roles?.isNotEmpty() == true) {
                            val guild = chatState.selectedGuild
                            Spacer(modifier = Modifier.height(16.dp))
                            Text(text = "ROLES", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(modifier = Modifier.height(8.dp))
                            
                            androidx.compose.foundation.layout.FlowRow(
                                horizontalArrangement = Arrangement.spacedBy(4.dp),
                                verticalArrangement = Arrangement.spacedBy(4.dp)
                            ) {
                                profile.guild_member.roles.mapNotNull { roleId -> guild?.roles?.find { it.id == roleId } }
                                    .sortedByDescending { it.position }
                                    .forEach { role ->
                                        Surface(
                                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f),
                                            shape = RoundedCornerShape(4.dp),
                                            border = if (role.color != 0) androidx.compose.foundation.BorderStroke(1.dp, Color(role.color or 0xFF000000.toInt()).copy(alpha = 0.5f)) else null
                                        ) {
                                            Row(
                                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                                                verticalAlignment = Alignment.CenterVertically
                                            ) {
                                                if (role.color != 0) {
                                                    Box(
                                                        modifier = Modifier
                                                            .size(12.dp)
                                                            .background(Color(role.color or 0xFF000000.toInt()), CircleShape)
                                                    )
                                                    Spacer(modifier = Modifier.width(6.dp))
                                                }
                                                Text(text = role.name, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Medium)
                                            }
                                        }
                                    }
                            }
                        }

                        if (profile.guild_member?.joined_at?.isNotBlank() == true) {
                            Spacer(modifier = Modifier.height(16.dp))
                            Text(text = "MEMBER SINCE", style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(text = profile.guild_member.joined_at.take(10), style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                }
                
                Spacer(modifier = Modifier.height(16.dp))
            }
        }
    }
}

object ModernProfileColors {
    fun luminanceOf(color: Int): Double {
        val r = (color shr 16) and 0xFF
        val g = (color shr 8) and 0xFF
        val b = color and 0xFF

        fun ch(x: Int): Double {
            val v = x / 255.0
            return if (v <= 0.03928) v / 12.92 else ((v + 0.055) / 1.055).pow(2.4)
        }
        return 0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
    }

    fun mix(a: Int, b: Int, t: Double): Int {
        val ar = (a shr 16) and 0xFF
        val ag = (a shr 8) and 0xFF
        val ab = a and 0xFF
        val br = (b shr 16) and 0xFF
        val bg = (b shr 8) and 0xFF
        val bb = b and 0xFF
        val rr = kotlin.math.round(ar * (1 - t) + br * t).toInt().coerceIn(0, 255)
        val rg = kotlin.math.round(ag * (1 - t) + bg * t).toInt().coerceIn(0, 255)
        val rb = kotlin.math.round(ab * (1 - t) + bb * t).toInt().coerceIn(0, 255)
        return (0xFF shl 24) or (rr shl 16) or (rg shl 8) or rb
    }

    fun softenColor(color: Int): Int {
        val lum = luminanceOf(color)
        return when {
            lum >= 0.85 -> mix(color, 0xFFFFFFFF.toInt(), 0.04)
            lum >= 0.6 -> mix(color, 0xFFFFFFFF.toInt(), 0.06)
            lum >= 0.3 -> mix(color, 0xFF000000.toInt(), 0.06)
            else -> mix(color, 0xFFFFFFFF.toInt(), 0.12)
        }
    }
}
