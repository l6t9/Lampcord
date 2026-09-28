package me.lampu.lampcord.shared.ui.components

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.api.UserApi
import me.lampu.lampcord.shared.model.User
import me.lampu.lampcord.shared.state.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.ui.kit.clickableCursor

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UserStatusBottomSheet(
    onDismiss: () -> Unit,
    onSwitchAccount: (SavedAccount) -> Unit,
    onSetCustomStatus: () -> Unit,
    onAddAccount: () -> Unit,
    userApi: UserApi = koinInject(),
    userStore: UserStore = koinInject(),
    tokenStore: TokenStore = koinInject(),
    presenceStore: PresenceStore = koinInject(),
    settingsStore: SettingsStore = koinInject()
) {
    val currentUser by userStore.currentUser.collectAsState()
    val accounts = tokenStore.getAccounts()
    val scope = rememberCoroutineScope()
    
    var showAccountSwitcher by remember { mutableStateOf(false) }

    AdaptiveModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 24.dp)
                .animateContentSize()
        ) {
            if (!showAccountSwitcher) {
                currentUser?.let { user ->
                    val currentStatus = presenceStore.getUserStatus(user.id, user.id, settingsStore.userSettings?.status)
                    
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(modifier = Modifier.size(40.dp)) {
                            AsyncImage(
                                model = CdnUrls.getUserAvatarUrl(user.id, user.avatar, 128),
                                contentDescription = null,
                                modifier = Modifier.fillMaxSize().clip(CircleShape)
                            )
                            Box(
                                modifier = Modifier
                                    .align(Alignment.BottomEnd)
                                    .size(14.dp)
                                    .background(MaterialTheme.colorScheme.surface, CircleShape)
                                    .padding(2.dp)
                            ) {
                                StatusIndicator(
                                    status = currentStatus,
                                    size = 10.dp,
                                    borderWidth = 0.dp
                                )
                            }
                        }
                        Spacer(Modifier.width(12.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = user.global_name ?: user.username ?: "Unknown",
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold
                            )
                            val customStatus = settingsStore.userSettings?.custom_status
                            if (customStatus != null && !customStatus.text.isNullOrBlank()) {
                                Text(
                                    text = customStatus.text,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis
                                )
                            } else {
                                Text(
                                    text = user.username ?: "",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }
                        
                        IconButton(onClick = onDismiss) {
                            Icon(Icons.Rounded.Close, null)
                        }
                    }
                }

                HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

                StatusMenuItem(
                    icon = { StatusIndicator(status = "online", size = 20.dp, borderWidth = 0.dp) },
                    label = "Online",
                    onClick = {
                        scope.launch { userApi.updateStatus("online") }
                        onDismiss()
                    }
                )
                StatusMenuItem(
                    icon = { StatusIndicator(status = "idle", size = 20.dp, borderWidth = 0.dp) },
                    label = "Idle",
                    onClick = {
                        scope.launch { userApi.updateStatus("idle") }
                        onDismiss()
                    }
                )
                StatusMenuItem(
                    icon = { StatusIndicator(status = "dnd", size = 20.dp, borderWidth = 0.dp) },
                    label = "Do Not Disturb",
                    onClick = {
                        scope.launch { userApi.updateStatus("dnd") }
                        onDismiss()
                    }
                )
                StatusMenuItem(
                    icon = { StatusIndicator(status = "invisible", size = 20.dp, borderWidth = 0.dp) },
                    label = "Invisible",
                    onClick = {
                        scope.launch { userApi.updateStatus("invisible") }
                        onDismiss()
                    }
                )

                HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp), color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))

                StatusMenuItem(
                    icon = { Icon(Icons.Rounded.EmojiEmotions, null, tint = MaterialTheme.colorScheme.onSurfaceVariant) },
                    label = "Set Custom Status",
                    onClick = {
                        onSetCustomStatus()
                        onDismiss()
                    }
                )

                StatusMenuItem(
                    icon = { Icon(Icons.Rounded.Groups, null, tint = MaterialTheme.colorScheme.onSurfaceVariant) },
                    label = "Switch Account",
                    onClick = { showAccountSwitcher = true },
                    trailingContent = if (accounts.size > 1) {
                        {
                            Text(
                                text = "${accounts.size}",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onPrimary,
                                modifier = Modifier
                                    .background(MaterialTheme.colorScheme.primary, CircleShape)
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        }
                    } else null
                )
            } else {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IconButton(onClick = { showAccountSwitcher = false }) {
                        Icon(Icons.AutoMirrored.Rounded.ArrowBack, null)
                    }
                    Text(
                        text = "Switch Account",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(start = 8.dp)
                    )
                }
                
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp)
                        .heightIn(max = 400.dp)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    accounts.forEach { account ->
                        val isSelected = account.user.id == currentUser?.id
                        AccountSwitcherItem(
                            account = account,
                            isSelected = isSelected,
                            onClick = {
                                if (!isSelected) {
                                    onSwitchAccount(account)
                                    onDismiss()
                                }
                            }
                        )
                    }
                    
                    Spacer(Modifier.height(8.dp))
                    
                    Button(
                        onClick = {
                            onAddAccount()
                            onDismiss()
                        },
                        modifier = Modifier.fillMaxWidth().height(52.dp),
                        shape = RoundedCornerShape(16.dp),
                    ) {
                        Icon(Icons.Filled.Add, null, modifier = Modifier.size(20.dp))
                        Spacer(Modifier.width(12.dp))
                        Text("Add Account")
                    }
                }
            }
        }
    }
}

@Composable
private fun AccountSwitcherItem(
    account: SavedAccount,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        shape = RoundedCornerShape(16.dp),
        color = if (isSelected) MaterialTheme.colorScheme.surfaceContainerHighest else MaterialTheme.colorScheme.surfaceContainerLow,
        border = if (isSelected) BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant) else null
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(modifier = Modifier.size(40.dp)) {
                AsyncImage(
                    model = CdnUrls.getUserAvatarUrl(account.user.id, account.user.avatar, 128),
                    contentDescription = null,
                    modifier = Modifier.fillMaxSize().clip(CircleShape)
                )
            }
            
            Spacer(modifier = Modifier.width(16.dp))
            
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = account.user.global_name ?: account.user.username ?: "Unknown User",
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = account.user.username ?: "",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            
            if (isSelected) {
                Icon(
                    imageVector = Icons.Rounded.CheckCircle,
                    contentDescription = "Selected",
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(24.dp)
                )
            }
        }
    }
}

@Composable
private fun StatusMenuItem(
    icon: @Composable () -> Unit,
    label: String,
    onClick: () -> Unit,
    trailingContent: @Composable (() -> Unit)? = null
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickableCursor(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(modifier = Modifier.size(24.dp), contentAlignment = Alignment.Center) {
            icon()
        }
        Spacer(Modifier.width(16.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            modifier = Modifier.weight(1f)
        )
        trailingContent?.invoke()
    }
}
