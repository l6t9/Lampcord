package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.account
import me.lampu.lampcord.shared.generated.resources.change_password
import me.lampu.lampcord.shared.generated.resources.edit
import me.lampu.lampcord.shared.generated.resources.email
import me.lampu.lampcord.shared.generated.resources.form_label_account_information
import me.lampu.lampcord.shared.generated.resources.form_label_username
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_password_auth
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_two_fa_disabled
import me.lampu.lampcord.shared.generated.resources.lampcord_settings_two_fa_enabled
import me.lampu.lampcord.shared.generated.resources.not_set
import me.lampu.lampcord.shared.generated.resources.two_fa
import me.lampu.lampcord.shared.generated.resources.unknown_user
import me.lampu.lampcord.shared.generated.resources.user_settings_avatar
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

@Composable
fun AccountSettings(
    onBack: () -> Unit,
    userStore: UserStore = koinInject()
) {
    val user by userStore.currentUser.collectAsState()
    val userVal = user ?: return
    
    SettingsSubScreen(
        title = stringResource(Res.string.account),
        onNavigateBack = onBack
    ) {
        AccountSettingsContent(userVal = userVal)
    }
}

@Composable
fun AccountSettingsContent(userVal: me.lampu.lampcord.shared.model.User) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Material3SettingsGroup(
            title = stringResource(Res.string.form_label_account_information),
            items = listOf(
                Material3SettingsItem(
                    leadingContent = {
                        AsyncImage(
                            model = CdnUrls.getUserAvatarUrl(userVal.id, userVal.avatar, 128),
                            contentDescription = stringResource(Res.string.user_settings_avatar),
                            modifier = Modifier.size(60.dp).clip(CircleShape)
                        )
                    },
                    title = { Text(userVal.global_name ?: userVal.username ?: stringResource(Res.string.unknown_user), fontWeight = FontWeight.Bold) },
                    description = { Text(userVal.username ?: "") },
                    trailingContent = {
                        Button(onClick = { /* TODO */ }) {
                            Text(stringResource(Res.string.edit))
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            items = listOf(
                Material3SettingsItem(
                    title = { Text(stringResource(Res.string.form_label_username)) },
                    description = { Text(userVal.username ?: stringResource(Res.string.not_set)) }
                ),
                Material3SettingsItem(
                    title = { Text(stringResource(Res.string.email)) },
                    description = { Text(userVal.email ?: stringResource(Res.string.not_set)) }
                )
            )
        )

        Material3SettingsGroup(
            title = stringResource(Res.string.lampcord_settings_password_auth),
            items = listOf(
                Material3SettingsItem(
                    title = { Text(stringResource(Res.string.change_password)) },
                    onClick = { /* TODO */ }
                ),
                Material3SettingsItem(
                    title = { Text(stringResource(Res.string.two_fa)) },
                    description = { 
                        Text(
                            text = stringResource(
                                if (userVal.mfa_enabled == true) Res.string.lampcord_settings_two_fa_enabled else Res.string.lampcord_settings_two_fa_disabled
                            ),
                            color = if (userVal.mfa_enabled == true) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    },
                    onClick = { /* TODO */ }
                )
            )
        )
    }
}
