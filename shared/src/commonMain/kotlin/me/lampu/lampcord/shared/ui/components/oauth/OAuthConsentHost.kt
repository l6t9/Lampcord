package me.lampu.lampcord.shared.ui.components.oauth

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.authorize
import me.lampu.lampcord.shared.generated.resources.oauth_consent_deny
import me.lampu.lampcord.shared.generated.resources.oauth_consent_error
import me.lampu.lampcord.shared.generated.resources.oauth_consent_redirect
import me.lampu.lampcord.shared.generated.resources.oauth_consent_title
import me.lampu.lampcord.shared.generated.resources.oauth_scope_identify
import me.lampu.lampcord.shared.generated.resources.oauth_scope_openid
import me.lampu.lampcord.shared.generated.resources.oauth_scope_sdk_social_layer
import me.lampu.lampcord.shared.generated.resources.oauth_scope_sdk_social_layer_presence
import me.lampu.lampcord.shared.generated.resources.oauth_scope_unknown
import me.lampu.lampcord.shared.generated.resources.oauth2_requests_account_access
import me.lampu.lampcord.shared.generated.resources.oauth2_scopes_label
import me.lampu.lampcord.shared.oauth.OAuthAuthorizationHandler
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.rememberLinkOpener
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

@Composable
fun OAuthConsentHost(handler: OAuthAuthorizationHandler = koinInject()) {
    val state by handler.state.collectAsState()
    val linkOpener = rememberLinkOpener()

    LaunchedEffect(handler) {
        handler.openRedirect = { url -> linkOpener(url) }
    }

    val request = state.request ?: return
    key(request.id) {
        AlertDialog(
            onDismissRequest = { handler.deny() },
            title = { Text(stringResource(Res.string.oauth_consent_title)) },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        val iconUrl = state.appIconUrl
                        if (iconUrl != null) {
                            AsyncImage(
                                model = iconUrl,
                                contentDescription = null,
                                modifier = Modifier.size(40.dp),
                                shape = CircleShape,
                            )
                        }
                        Text(
                            text = "${state.appName ?: request.clientId} " +
                                stringResource(Res.string.oauth2_requests_account_access),
                            style = MaterialTheme.typography.titleMedium,
                        )
                    }
                    state.appDescription?.takeIf { it.isNotBlank() }?.let { description ->
                        Text(
                            text = description,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Text(
                        text = stringResource(Res.string.oauth2_scopes_label),
                        style = MaterialTheme.typography.titleSmall,
                    )
                    request.scopes.forEach { scope ->
                        Text(
                            text = "• ${scopeLabel(scope)}",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                    state.redirectUri?.let { uri ->
                        Text(
                            text = stringResource(Res.string.oauth_consent_redirect, uri),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    state.error?.let { error ->
                        Text(
                            text = stringResource(Res.string.oauth_consent_error, error),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            },
            confirmButton = {
                Button(onClick = { handler.approve() }, enabled = !state.isLoading) {
                    Text(stringResource(Res.string.authorize))
                }
            },
            dismissButton = {
                TextButton(onClick = { handler.deny() }, enabled = !state.isLoading) {
                    Text(stringResource(Res.string.oauth_consent_deny))
                }
            },
        )
    }
}

@Composable
private fun scopeLabel(scope: String): String = when (scope) {
    "openid" -> stringResource(Res.string.oauth_scope_openid)
    "identify" -> stringResource(Res.string.oauth_scope_identify)
    "sdk.social_layer_presence" -> stringResource(Res.string.oauth_scope_sdk_social_layer_presence)
    "sdk.social_layer" -> stringResource(Res.string.oauth_scope_sdk_social_layer)
    else -> stringResource(Res.string.oauth_scope_unknown)
}
