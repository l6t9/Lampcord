package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.draw.clip
import androidx.compose.foundation.shape.CircleShape
import me.lampu.lampcord.shared.state.ChatState
import me.lampu.lampcord.shared.api.RemoteAuthClient
import me.lampu.lampcord.shared.api.RemoteAuthState
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.MeshGradientBackground
import kotlinx.coroutines.launch
import org.koin.compose.koinInject

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun LoginScreen(
    chatState: ChatState = koinInject(),
    remoteAuthClient: RemoteAuthClient = koinInject(),
    onLoginSuccess: () -> Unit
) {
    var login by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var mfaCode by remember { mutableStateOf("") }
    var mfaTicket by remember { mutableStateOf<String?>(null) }
    var mfaType by remember { mutableStateOf("totp") }
    var isLoading by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    
    val scope = rememberCoroutineScope()
    val remoteAuthState by remoteAuthClient.state.collectAsState()

    LaunchedEffect(remoteAuthState) {
        val state = remoteAuthState
        if (state is RemoteAuthState.Finished) {
            chatState.connect(state.token)
            onLoginSuccess()
        }
    }

    LaunchedEffect(Unit) {
        remoteAuthClient.start()
    }

    DisposableEffect(Unit) {
        onDispose {
            remoteAuthClient.stop()
        }
    }

    fun performLogin() {
        scope.launch {
            isLoading = true
            errorMessage = null
            if (mfaTicket == null) {
                val response = chatState.login(login, password)
                if (response == null) {
                    errorMessage = "Login failed"
                } else if (response.token != null) {
                    chatState.connect(response.token)
                    onLoginSuccess()
                } else if (response.mfa == true && response.ticket != null) {
                    mfaTicket = response.ticket
                } else if (response.message != null) {
                    errorMessage = response.message
                }
            } else {
                val currentTicket: String? = mfaTicket
                if (currentTicket != null) {
                    val res = chatState.verifyMFA(mfaCode.trim(), currentTicket, mfaType)
                    if (res) {
                        onLoginSuccess()
                    } else {
                        errorMessage = "Invalid code or expired ticket"
                    }
                }
            }
            isLoading = false
        }
    }

    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background
    ) {
        MeshGradientBackground()

        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            Column(
                modifier = Modifier
                    .widthIn(max = 480.dp)
                    .fillMaxWidth()
                    .padding(32.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Text(
                    text = "          へ   ♡     ╱|、\n" +
                           "     ૮  -   ՛ )      (`   -  7\n" +
                           "       /   ⁻  ៸|       |、⁻〵\n" +
                           " 乀 (ˍ, ل ل        じしˍ,)ノ",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.primary,
                    lineHeight = 22.sp
                )

                Spacer(modifier = Modifier.height(32.dp))

                if (mfaTicket == null) {
                    TextField(
                        value = login,
                        onValueChange = { login = it },
                        label = { Text("Email or Phone Number") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = MaterialTheme.shapes.large,
                        colors = TextFieldDefaults.colors(
                            unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                            focusedContainerColor = MaterialTheme.colorScheme.surfaceVariant
                        ),
                        keyboardOptions = KeyboardOptions(
                            keyboardType = KeyboardType.Email,
                            imeAction = ImeAction.Next
                        )
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    TextField(
                        value = password,
                        onValueChange = { password = it },
                        label = { Text("Password") },
                        visualTransformation = PasswordVisualTransformation(),
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = MaterialTheme.shapes.large,
                        colors = TextFieldDefaults.colors(
                            unfocusedContainerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                            focusedContainerColor = MaterialTheme.colorScheme.surfaceVariant
                        ),
                        keyboardOptions = KeyboardOptions(
                            keyboardType = KeyboardType.Password,
                            imeAction = ImeAction.Done
                        ),
                        keyboardActions = KeyboardActions(
                            onDone = { performLogin() }
                        )
                    )
                } else {
                    Text(
                        if (mfaType == "totp") "Enter 2FA Code" else "Enter Backup Code",
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    TextField(
                        value = mfaCode,
                        onValueChange = { mfaCode = it },
                        label = { Text(if (mfaType == "totp") "6-digit code" else "8-digit backup code") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        shape = MaterialTheme.shapes.large,
                        keyboardOptions = KeyboardOptions(
                            keyboardType = if (mfaType == "totp") KeyboardType.Number else KeyboardType.Text,
                            imeAction = ImeAction.Done
                        ),
                        keyboardActions = KeyboardActions(
                            onDone = { performLogin() }
                        )
                    )
                    
                    TextButton(onClick = { 
                        mfaType = if (mfaType == "totp") "backup" else "totp" 
                        mfaCode = ""
                    }) {
                        Text(if (mfaType == "totp") "Use Backup Code" else "Use Authenticator App")
                    }
                }

                if (errorMessage != null) {
                    Spacer(modifier = Modifier.height(16.dp))
                    Surface(
                        color = MaterialTheme.colorScheme.errorContainer,
                        shape = MaterialTheme.shapes.medium,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text(
                            text = errorMessage!!,
                            color = MaterialTheme.colorScheme.onErrorContainer,
                            modifier = Modifier.padding(12.dp),
                            style = MaterialTheme.typography.labelMedium
                        )
                    }
                }

                Spacer(modifier = Modifier.height(32.dp))

                Button(
                    onClick = { performLogin() },
                    enabled = !isLoading,
                    modifier = Modifier.fillMaxWidth().height(56.dp),
                    shape = MaterialTheme.shapes.large
                ) {
                    if (isLoading) {
                        // Contained loading indicator themed for the filled login button.
                        ContainedLoadingIndicator(
                            modifier = Modifier.size(24.dp),
                            containerColor = MaterialTheme.colorScheme.onPrimary,
                            indicatorColor = MaterialTheme.colorScheme.primary,
                        )
                    } else {
                        Text(
                            text = if (mfaTicket == null) "Login" else "Verify",
                            style = MaterialTheme.typography.titleMedium
                        )
                    }
                }
                
                if (mfaTicket != null) {
                    TextButton(
                        onClick = { mfaTicket = null; mfaCode = "" },
                        modifier = Modifier.padding(top = 8.dp)
                    ) {
                        Text("Back to Login")
                    }
                }

                if (mfaTicket == null) {
                    Spacer(modifier = Modifier.height(32.dp))
                    
                    HorizontalDivider(
                        modifier = Modifier.fillMaxWidth(),
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f)
                    )
                    
                    Spacer(modifier = Modifier.height(32.dp))
                    
                    Text(
                        text = "Or scan this QR code to log in instantly",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Surface(
                        modifier = Modifier.size(200.dp),
                        shape = MaterialTheme.shapes.large,
                        color = Color.White,
                        tonalElevation = 2.dp
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            when (val state = remoteAuthState) {
                                is RemoteAuthState.QRReady -> {
                                    val qrUrl = "https://api.qrserver.com/v1/create-qr-code/?size=256x256&data=${state.url}"
                                    me.lampu.lampcord.shared.ui.components.AsyncImage(
                                        model = qrUrl,
                                        contentDescription = "QR Code",
                                        modifier = Modifier.fillMaxSize().padding(12.dp)
                                    )
                                }
                                is RemoteAuthState.UserScanned -> {
                                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                        val avatarUrl = state.user.avatar?.let {
                                            "https://cdn.discordapp.com/avatars/${state.user.id}/$it.png?size=128"
                                        }
                                        if (avatarUrl != null) {
                                            me.lampu.lampcord.shared.ui.components.AsyncImage(
                                                model = avatarUrl,
                                                contentDescription = "Avatar",
                                                modifier = Modifier.size(64.dp).clip(CircleShape)
                                            )
                                        }
                                        Spacer(Modifier.height(8.dp))
                                        Text(
                                            state.user.global_name ?: state.user.username,
                                            style = MaterialTheme.typography.titleSmall,
                                            color = MaterialTheme.colorScheme.onSurface
                                        )
                                        Text(
                                            "Approve on your phone",
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }
                                is RemoteAuthState.Connecting -> {
                                    CircularProgressIndicator(modifier = Modifier.size(32.dp))
                                }
                                is RemoteAuthState.Error -> {
                                    Column(
                                        horizontalAlignment = Alignment.CenterHorizontally,
                                        modifier = Modifier.padding(16.dp)
                                    ) {
                                        Text("Error", color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.labelLarge)
                                        TextButton(onClick = { scope.launch { remoteAuthClient.start() } }) {
                                            Text("Retry")
                                        }
                                    }
                                }
                                else -> {
                                    CircularProgressIndicator(modifier = Modifier.size(32.dp))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
