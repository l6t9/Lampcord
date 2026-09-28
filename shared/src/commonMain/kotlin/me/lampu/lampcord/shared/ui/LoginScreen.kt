package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.draw.clip
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.text.input.VisualTransformation
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.api.RemoteAuthClient
import me.lampu.lampcord.shared.api.RemoteAuthState
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject
import qrcode.QRCode
import qrcode.raw.ErrorCorrectionLevel
import kotlin.math.min

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun LoginScreen(
    modifier: Modifier = Modifier,
    sessionManager: SessionManager = koinInject(),
    remoteAuthClient: RemoteAuthClient = koinInject(),
    onLoginSuccess: () -> Unit
) {
    var login by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var passwordVisible by remember { mutableStateOf(false) }
    var useTokenLogin by remember { mutableStateOf(false) }
    var token by remember { mutableStateOf("") }
    var tokenVisible by remember { mutableStateOf(false) }
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
            sessionManager.connect(state.token)
            onLoginSuccess()
        }
    }

    LaunchedEffect(useTokenLogin) {
        if (useTokenLogin) {
            remoteAuthClient.stop()
        } else {
            remoteAuthClient.start()
        }
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
            try {
                if (mfaTicket == null) {
                    if (useTokenLogin) {
                        val authToken = token.trim()
                        if (authToken.isBlank()) {
                            errorMessage = "Enter a token"
                        } else {
                            sessionManager.connect(authToken)
                            onLoginSuccess()
                        }
                    } else {
                        val response = sessionManager.login(login, password)
                        if (response == null) {
                            errorMessage = "Login failed (check your connection)"
                        } else if (response.token != null) {
                            sessionManager.connect(response.token)
                            onLoginSuccess()
                        } else if (response.mfa == true && response.ticket != null) {
                            mfaTicket = response.ticket
                        } else if (response.message != null) {
                            errorMessage = response.message
                        } else {
                            errorMessage = "Invalid login or password"
                        }
                    }
                } else {
                    val currentTicket: String? = mfaTicket
                    if (currentTicket != null) {
                        val res = sessionManager.verifyMFA(mfaCode.trim(), currentTicket, mfaType)
                        if (res) {
                            onLoginSuccess()
                        } else {
                            errorMessage = "Invalid code or expired ticket"
                        }
                    }
                }
            } catch (e: Exception) {
                errorMessage = e.message ?: "An unexpected error occurred"
            } finally {
                isLoading = false
            }
        }
    }

    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background
    ) {
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
                    if (useTokenLogin) {
                        TextField(
                            value = token,
                            onValueChange = { token = it },
                            label = { Text("Discord Token") },
                            visualTransformation = if (tokenVisible) VisualTransformation.None else PasswordVisualTransformation(),
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
                            ),
                            trailingIcon = {
                                IconButton(onClick = { tokenVisible = !tokenVisible }) {
                                    Icon(
                                        imageVector = if (tokenVisible) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                                        contentDescription = if (tokenVisible) "Hide token" else "Show token"
                                    )
                                }
                            }
                        )
                    } else {
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
                            visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
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
                            ),
                            trailingIcon = {
                                IconButton(onClick = { passwordVisible = !passwordVisible }) {
                                    Icon(
                                        imageVector = if (passwordVisible) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                                        contentDescription = if (passwordVisible) "Hide password" else "Show password"
                                    )
                                }
                            }
                        )
                    }

                    TextButton(
                        onClick = {
                            useTokenLogin = !useTokenLogin
                            errorMessage = null
                        },
                        modifier = Modifier.padding(top = 4.dp)
                    ) {
                        Text(if (useTokenLogin) "Use email and password instead" else "Login with token")
                    }
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

                if (mfaTicket == null && !useTokenLogin) {
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
                                    val qrCode = remember(state.url) {
                                        QRCode.ofSquares()
                                            .withErrorCorrectionLevel(ErrorCorrectionLevel.MEDIUM)
                                            .build(state.url)
                                    }

                                    Canvas(
                                        modifier = Modifier.fillMaxSize().padding(8.dp)
                                    ) {
                                        drawLocalQrCode(qrCode)
                                    }
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
                                    ContainedLoadingIndicator(modifier = Modifier.size(32.dp))
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
                                    ContainedLoadingIndicator(modifier = Modifier.size(32.dp))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawLocalQrCode(qrCode: QRCode) {
    drawRect(color = Color.White)

    val quietZone = 4
    val moduleCount = qrCode.rawData.size
    val totalModules = moduleCount + quietZone * 2
    val moduleSize = min(size.width, size.height) / totalModules
    val codeSize = totalModules * moduleSize
    val offsetX = (size.width - codeSize) / 2f
    val offsetY = (size.height - codeSize) / 2f

    qrCode.rawData.forEach { cells ->
        cells.forEach { cell ->
            if (cell.dark) {
                drawRect(
                    color = Color.Black,
                    topLeft = Offset(
                        x = offsetX + (cell.col + quietZone) * moduleSize,
                        y = offsetY + (cell.row + quietZone) * moduleSize,
                    ),
                    size = Size(moduleSize, moduleSize),
                )
            }
        }
    }
}
