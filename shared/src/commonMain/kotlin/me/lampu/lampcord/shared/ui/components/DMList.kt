package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun DMList(
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject()
) {
    val scrollState = rememberLazyListState()
    var isHovered by remember { mutableStateOf(false) }
    val privateChannels by guildStore.privateChannels.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent()
                        when (event.type) {
                            PointerEventType.Enter -> isHovered = true
                            PointerEventType.Exit -> isHovered = false
                        }
                    }
                }
            }
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
                .padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "Direct Messages",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.weight(1f)
            )

            if (me.lampu.lampcord.shared.utils.getPlatformName() != "android" && me.lampu.lampcord.shared.utils.getPlatformName() != "ios") {
                IconButton(
                    onClick = { navigationStore.selectFriends() },
                    modifier = Modifier.size(32.dp)
                ) {
                    Icon(
                        imageVector = if (navigationStore.isFriendsSelected) Icons.Filled.Person else Icons.Rounded.Person,
                        contentDescription = "Friends",
                        tint = if (navigationStore.isFriendsSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
        }
        
        Box(modifier = Modifier.fillMaxSize()) {
            LazyColumn(
                state = scrollState,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(top = 8.dp),
                contentPadding = PaddingValues(bottom = 68.dp)
            ) {
                if (privateChannels.isEmpty()) {
                    items(10) {
                        DMSkeleton()
                    }
                } else {
                    items(privateChannels.distinctBy { it.id }, key = { it.id }) { channel ->
                        DMItem(channel)
                    }
                }
            }

            VerticalScrollbar(
                state = scrollState,
                modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight(),
                isVisible = isHovered
            )
        }
    }
}

@Composable
fun DMSkeleton() {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(48.dp)
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        ShimmerBox(
            modifier = Modifier.size(32.dp),
            shape = androidx.compose.foundation.shape.CircleShape
        )
        Spacer(modifier = Modifier.width(12.dp))
        ShimmerBox(
            modifier = Modifier
                .width(100.dp)
                .height(14.dp),
            shape = RoundedCornerShape(7.dp)
        )
    }
}
