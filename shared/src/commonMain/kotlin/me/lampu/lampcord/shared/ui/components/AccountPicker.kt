package me.lampu.lampcord.shared.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.state.SavedAccount

@Composable
fun AccountPicker(
    chatState: ChatState,
    onAccountSelected: (SavedAccount) -> Unit
) {
    val accounts = chatState.tokenStore.getAccounts()
    val currentUser = chatState.currentUser

    Surface(
        modifier = Modifier.width(320.dp).wrapContentHeight(),
        shape = RoundedCornerShape(12.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        shadowElevation = 8.dp
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Text(
                "Switch Account",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 12.dp)
            )

            accounts.forEach { account ->
                val isSelected = account.user.id == currentUser?.id
                AccountItem(
                    account = account,
                    isSelected = isSelected,
                    onClick = { onAccountSelected(account) }
                )
                Spacer(modifier = Modifier.height(8.dp))
            }
            
            HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
            
            TextButton(
                onClick = { /* TODO: Add Account flow */ },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Add another account")
            }
        }
    }
}

@Composable
private fun AccountItem(
    account: SavedAccount,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(8.dp),
        color = if (isSelected) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceContainerLow
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val avatarUrl = account.user.avatar?.let {
                "https://cdn.discordapp.com/avatars/${account.user.id}/$it.png?size=128"
            }

            Box(modifier = Modifier.size(32.dp)) {
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
                            filterQuality = FilterQuality.Medium
                        )
                    } else {
                        Box(contentAlignment = Alignment.Center) {
                            Text(account.user.username.take(1).uppercase(), style = MaterialTheme.typography.labelMedium)
                        }
                    }
                }
            }
            
            Spacer(modifier = Modifier.width(12.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = account.user.global_name ?: account.user.username,
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = account.user.username,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            
            if (isSelected) {
                RadioButton(selected = true, onClick = null)
            }
        }
    }
}
