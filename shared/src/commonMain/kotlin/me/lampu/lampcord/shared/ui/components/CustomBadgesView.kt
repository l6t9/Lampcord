package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.BadgeStore
import me.lampu.lampcord.shared.utils.ResourceLoader
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomBadgesView(
    userId: String,
    modifier: Modifier = Modifier,
    badgeSize: Dp = 16.dp,
    spacing: Dp = 4.dp,
    badgeStore: BadgeStore = koinInject()
) {
    val lampcordBadges by badgeStore.lampcordBadges.collectAsState()

    val badges = remember(userId, lampcordBadges) {
        badgeStore.getUserBadges(userId)
    }

    if (badges.isEmpty()) return

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(spacing),
        modifier = modifier
    ) {
        badges.forEach { badge ->
            badge.icon?.let { icon ->
                val model = remember(icon) {
                    if (icon.startsWith("http://") || icon.startsWith("https://") ||
                        icon.startsWith("file://") || icon.startsWith("content://")
                    ) {
                        icon
                    } else {
                        ResourceLoader.readBytes(icon)
                    }
                }
                ExpressiveTooltip(
                    anchorPosition = TooltipAnchorPosition.Above,
                    content = tooltipText(badge.name),
                    anchor = {
                        AsyncImage(
                            model = model,
                            contentDescription = badge.name,
                            modifier = Modifier
                                .size(badgeSize)
                                .clickableCursor { showToast(badge.name) }
                        )
                    }
                )
            }
        }
    }
}
