package me.lampu.lampcord.shared.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.settings.Settings
import me.lampu.lampcord.shared.settings.ThemeMode
import me.lampu.lampcord.shared.state.SavedAccount
import me.lampu.lampcord.shared.state.SessionManager
import me.lampu.lampcord.shared.state.SettingsStore
import me.lampu.lampcord.shared.state.TokenStore
import me.lampu.lampcord.shared.ui.components.AccountSwitcherItem
import me.lampu.lampcord.shared.ui.components.ExpressiveSwitch
import me.lampu.lampcord.shared.ui.components.settings.SettingsButtonGroup
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

private enum class OnboardingStep(
    val title: String,
    val description: String,
    val icon: ImageVector?,
    val primaryLabel: String,
    val secondaryLabel: String? = null
) {
    Welcome(
        title = "Welcome to Lampcord",
        description = "A modern Discord client. Let's get a few things set up for you.",
        icon = null,
        primaryLabel = "Get started"
    ),
    Appearance(
        title = "Appearance",
        description = "Choose how Lampcord looks. You can always fine-tune this later in Settings.",
        icon = Icons.Filled.Palette,
        primaryLabel = "Continue"
    ),
    Basics(
        title = "Preferences",
        description = "Pick a few basics. Everything here can be changed anytime in Settings.",
        icon = Icons.Filled.Tune,
        primaryLabel = "Continue"
    ),
    Accounts(
        title = "Accounts",
        description = "Add an account to sign in now, or continue to sign in later from the login screen.",
        icon = Icons.Filled.Person,
        primaryLabel = "Continue"
    ),
    Finish(
        title = "You're all set",
        description = "Your preferences are saved. You can adjust them anytime in Settings.",
        icon = Icons.Filled.Check,
        primaryLabel = "Open Lampcord"
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class, ExperimentalMaterial3Api::class)
@Composable
fun OnboardingScreen(
    modifier: Modifier = Modifier,
    tokenStore: TokenStore = koinInject(),
    sessionManager: SessionManager = koinInject(),
    settingsStore: SettingsStore = koinInject(),
    onCompleted: () -> Unit
) {
    val steps = remember { OnboardingStep.entries }
    var stepIndex by remember { mutableIntStateOf(0) }
    var showAddAccountLogin by remember { mutableStateOf(false) }
    var selectedAccountToken by remember { mutableStateOf<String?>(null) }
    var pendingLoginAccountToken by remember { mutableStateOf<String?>(null) }

    fun complete() {
        val tokenToLogin = pendingLoginAccountToken
        Settings.shared.onboardingCompleted = true
        if (!tokenToLogin.isNullOrBlank()) {
            sessionManager.switchAccount(tokenToLogin)
        }
        onCompleted()
    }

    fun next() {
        if (stepIndex < steps.lastIndex) stepIndex++ else complete()
    }

    fun continueFromAccounts() {
        val accounts = tokenStore.getAccounts()
        pendingLoginAccountToken = if (accounts.isEmpty()) {
            null
        } else {
            (accounts.firstOrNull { it.token == selectedAccountToken } ?: accounts.first()).token
        }
        next()
    }

    fun goBack() {
        if (showAddAccountLogin) {
            showAddAccountLogin = false
        } else if (stepIndex > 0) {
            stepIndex--
        }
    }

    if (showAddAccountLogin) {
        Box(modifier = modifier.fillMaxSize()) {
            LoginScreen(
                connectOnSuccess = false,
                onLoginSuccess = {
                    showAddAccountLogin = false
                }
            )
            FilledTonalIconButton(
                onClick = { showAddAccountLogin = false },
                modifier = Modifier
                    .statusBarsPadding()
                    .padding(12.dp)
                    .align(Alignment.TopStart)
            ) {
                Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back")
            }
        }
        return
    }

    val step = steps[stepIndex]
    val isWelcome = step == OnboardingStep.Welcome

    Surface(
        modifier = modifier.fillMaxSize(),
        color = if (isWelcome) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.surface
    ) {
        BoxWithConstraints(
            Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding()
                .clipToBounds()
        ) {
            val wide = maxWidth >= 840.dp && maxWidth > maxHeight
            val gutter = if (wide) 56.dp else 24.dp
            val stageWidth = minOf(maxWidth, 600.dp)
            val availableHeight = maxHeight

            // Subtle welcome accent background
            if (isWelcome) {
                Box(
                    Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.surface)
                ) {
                    Box(
                        Modifier
                            .size(420.dp)
                            .offset(x = stageWidth * 0.25f, y = (-120).dp)
                            .align(Alignment.TopEnd)
                            .background(
                                MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.35f),
                                shape = RoundedCornerShape(220.dp)
                            )
                    )
                    Box(
                        Modifier
                            .size(300.dp)
                            .offset(x = (-80).dp, y = 80.dp)
                            .align(Alignment.BottomStart)
                            .background(
                                MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.25f),
                                shape = RoundedCornerShape(180.dp)
                            )
                    )
                }
            }

            Column(
                Modifier
                    .width(stageWidth)
                    .fillMaxHeight()
                    .align(Alignment.Center)
            ) {
                Box(Modifier.weight(1f).fillMaxWidth().clipToBounds()) {
                    Column(Modifier.fillMaxSize()) {
                        if (isWelcome) {
                            Row(
                                Modifier
                                    .fillMaxWidth()
                                    .padding(start = gutter, end = gutter, top = 12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(10.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Brand.Discord,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onSurface,
                                    modifier = Modifier.size(24.dp)
                                )
                                Text("Lampcord", style = MaterialTheme.typography.titleMedium)
                            }
                        }

                        AnimatedContent(
                            targetState = stepIndex,
                            modifier = Modifier.fillMaxSize(),
                            transitionSpec = {
                                if (targetState > initialState) {
                                    (fadeIn(tween(220)) + slideInHorizontally(tween(260)) { it / 8 }) togetherWith
                                        (fadeOut(tween(180)) + slideOutHorizontally(tween(220)) { -it / 8 })
                                } else {
                                    (fadeIn(tween(220)) + slideInHorizontally(tween(260)) { -it / 8 }) togetherWith
                                        (fadeOut(tween(180)) + slideOutHorizontally(tween(220)) { it / 8 })
                                }
                            },
                            label = "OnboardingPage"
                        ) { idx ->
                            val s = steps[idx]
                            val welcomePage = s == OnboardingStep.Welcome
                            val scrollState = rememberScrollState()
                            BoxWithConstraints(Modifier.fillMaxSize()) {
                                val viewportHeight = maxHeight
                                Column(
                                    Modifier
                                        .fillMaxWidth()
                                        .verticalScroll(scrollState)
                                        .heightIn(min = viewportHeight)
                                        .padding(
                                            horizontal = gutter,
                                            vertical = if (availableHeight < 500.dp) 16.dp else 32.dp
                                        ),
                                    verticalArrangement = if (welcomePage) Arrangement.Bottom else Arrangement.Top
                                ) {
                                    if (!welcomePage && s.icon != null) {
                                        Icon(
                                            imageVector = s.icon,
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.primary,
                                            modifier = Modifier.size(32.dp)
                                        )
                                        Spacer(Modifier.height(24.dp))
                                    }

                                    Text(
                                        text = "Step ${idx + 1} of ${steps.size}",
                                        style = MaterialTheme.typography.labelLarge,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        modifier = Modifier.padding(bottom = 8.dp)
                                    )
                                    Text(
                                        text = s.title,
                                        style = MaterialTheme.typography.headlineLarge,
                                        fontWeight = FontWeight.Normal,
                                        fontSize = 32.sp,
                                        lineHeight = 40.sp,
                                        modifier = Modifier.widthIn(max = 540.dp)
                                    )
                                    Text(
                                        text = s.description,
                                        style = MaterialTheme.typography.bodyLarge,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        modifier = Modifier.padding(top = 16.dp).widthIn(max = 540.dp)
                                    )

                                    when (s) {
                                        OnboardingStep.Welcome -> { /* content is bottom-aligned hero text */ }
                                        OnboardingStep.Appearance -> AppearancePageContent(settingsStore)
                                        OnboardingStep.Basics -> BasicsPageContent(settingsStore)
                                        OnboardingStep.Accounts -> AccountsPageContent(
                                            tokenStore = tokenStore,
                                            selectedAccountToken = selectedAccountToken,
                                            onSelectAccount = { account -> selectedAccountToken = account.token },
                                            onAddAccount = { showAddAccountLogin = true }
                                        )
                                        OnboardingStep.Finish -> FinishPageContent()
                                    }
                                }
                            }
                        }
                    }
                }

                OnboardingBottomActions(
                    stepIndex = stepIndex,
                    stepCount = steps.size,
                    isWelcome = isWelcome,
                    primaryLabel = step.primaryLabel,
                    secondaryLabel = step.secondaryLabel,
                    canGoBack = stepIndex > 0,
                    onBack = { goBack() },
                    onPrimary = {
                        if (step == OnboardingStep.Accounts) continueFromAccounts() else next()
                    },
                    onSecondary = null,
                    gutter = gutter
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun OnboardingBottomActions(
    stepIndex: Int,
    stepCount: Int,
    isWelcome: Boolean,
    primaryLabel: String,
    secondaryLabel: String?,
    canGoBack: Boolean,
    onBack: () -> Unit,
    onPrimary: () -> Unit,
    onSecondary: (() -> Unit)?,
    gutter: androidx.compose.ui.unit.Dp
) {
    val secondaryWeight by animateFloatAsState(
        targetValue = if (secondaryLabel == null) 0f else 1f,
        animationSpec = tween(320),
        label = "onboardingSecondaryWeight"
    )
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = gutter, vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        if (canGoBack) {
            IconButton(onClick = onBack) {
                Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = "Back")
            }
        }

        if (!isWelcome) {
            if (secondaryLabel != null) {
                TextButton(
                    onClick = { onSecondary?.invoke() },
                    modifier = Modifier
                        .weight(secondaryWeight.coerceAtLeast(0.001f))
                        .heightIn(min = 56.dp)
                ) {
                    Text(secondaryLabel)
                }
            } else if (secondaryWeight > 0f) {
                Spacer(Modifier.weight(secondaryWeight))
            }
        }

        Button(
            onClick = onPrimary,
            modifier = Modifier
                .weight(if (secondaryLabel == null) 1f else (3f - secondaryWeight))
                .heightIn(min = if (isWelcome) 64.dp else 56.dp)
        ) {
            Text(primaryLabel)
            if (isWelcome) {
                Spacer(Modifier.width(8.dp))
                Icon(Icons.AutoMirrored.Rounded.ArrowForward, contentDescription = null)
            }
        }
    }
}

@Composable
private fun AppearancePageContent(settingsStore: SettingsStore) {
    var themeMode by remember { mutableStateOf(settingsStore.themeMode) }
    var materialYou by remember { mutableStateOf(settingsStore.materialYou) }
    var pureBlack by remember { mutableStateOf(settingsStore.pureBlack) }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 32.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("Theme", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
        SettingsButtonGroup<ThemeMode>(
            options = listOf(ThemeMode.AUTO, ThemeMode.LIGHT, ThemeMode.DARK),
            selectedOption = themeMode,
            onOptionSelected = { mode ->
                themeMode = mode
                settingsStore.themeMode = mode
            },
            iconProvider = { mode, isSelected ->
                when (mode) {
                    ThemeMode.AUTO -> if (isSelected) Icons.Filled.Sync else Icons.Rounded.Sync
                    ThemeMode.LIGHT -> if (isSelected) Icons.Filled.LightMode else Icons.Rounded.LightMode
                    ThemeMode.DARK -> if (isSelected) Icons.Filled.DarkMode else Icons.Rounded.DarkMode
                }
            },
            labelProvider = { mode ->
                when (mode) {
                    ThemeMode.AUTO -> "System"
                    ThemeMode.LIGHT -> "Light"
                    ThemeMode.DARK -> "Dark"
                }
            }
        )

        Spacer(Modifier.height(4.dp))
        OnboardingSwitchRow(
            title = "Material You",
            subtitle = "Use dynamic colors from your system when available",
            checked = materialYou,
            onCheckedChange = {
                materialYou = it
                settingsStore.materialYou = it
            }
        )
        OnboardingSwitchRow(
            title = "Pure black (OLED)",
            subtitle = "Use a true black background in dark mode",
            checked = pureBlack,
            onCheckedChange = {
                pureBlack = it
                settingsStore.pureBlack = it
            }
        )
    }
}

@Composable
private fun BasicsPageContent(settingsStore: SettingsStore) {
    var notificationsEnabled by remember { mutableStateOf(settingsStore.notificationsEnabled) }
    var showInAppNotifications by remember { mutableStateOf(settingsStore.showInAppNotifications) }
    var showMessagePreview by remember { mutableStateOf(settingsStore.showMessagePreview) }
    var reduceMotion by remember { mutableStateOf(Settings.shared.reduceMotion) }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 32.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        OnboardingSwitchRow(
            title = "Notifications",
            subtitle = "Receive notifications for messages and activity",
            checked = notificationsEnabled,
            onCheckedChange = {
                notificationsEnabled = it
                settingsStore.notificationsEnabled = it
            }
        )
        OnboardingSwitchRow(
            title = "In-app notifications",
            subtitle = "Show notifications while using Lampcord",
            checked = showInAppNotifications,
            onCheckedChange = {
                showInAppNotifications = it
                settingsStore.showInAppNotifications = it
            }
        )
        OnboardingSwitchRow(
            title = "Message preview",
            subtitle = "Show message content in notifications",
            checked = showMessagePreview,
            onCheckedChange = {
                showMessagePreview = it
                settingsStore.showMessagePreview = it
            }
        )
        OnboardingSwitchRow(
            title = "Reduce motion",
            subtitle = "Minimize animations and motion effects",
            checked = reduceMotion,
            onCheckedChange = {
                reduceMotion = it
                Settings.shared.reduceMotion = it
            }
        )
    }
}

@Composable
private fun AccountsPageContent(
    tokenStore: TokenStore,
    selectedAccountToken: String?,
    onSelectAccount: (SavedAccount) -> Unit,
    onAddAccount: () -> Unit
) {
    val accounts = tokenStore.getAccounts()
    val effectiveSelectedToken = selectedAccountToken?.takeIf { token -> accounts.any { it.token == token } }
        ?: accounts.firstOrNull()?.token

    LaunchedEffect(accounts, selectedAccountToken) {
        if (accounts.isNotEmpty() && effectiveSelectedToken != null && selectedAccountToken != effectiveSelectedToken) {
            accounts.firstOrNull { it.token == effectiveSelectedToken }?.let(onSelectAccount)
        }
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 32.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        if (accounts.isEmpty()) {
            Surface(
                shape = RoundedCornerShape(20.dp),
                color = MaterialTheme.colorScheme.surfaceContainerLow,
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier.padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Icon(
                        Icons.Filled.Person,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(28.dp)
                    )
                    Spacer(Modifier.height(12.dp))
                    Text(
                        "No accounts added yet",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.SemiBold
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        "Add an account to sign in now, or continue to sign in later.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        textAlign = TextAlign.Center
                    )
                }
            }
        } else {
            accounts.forEach { account ->
                AccountSwitcherItem(
                    account = account,
                    isSelected = account.token == effectiveSelectedToken,
                    onClick = { onSelectAccount(account) }
                )
            }
        }

        Button(
            onClick = onAddAccount,
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp),
            shape = RoundedCornerShape(16.dp)
        ) {
            Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.size(20.dp))
            Spacer(Modifier.width(8.dp))
            Text("Add account")
        }
    }
}

@Composable
private fun FinishPageContent() {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 32.dp),
        horizontalAlignment = Alignment.Start
    ) {
        Surface(
            shape = RoundedCornerShape(20.dp),
            color = MaterialTheme.colorScheme.surfaceContainerLow,
            modifier = Modifier.fillMaxWidth()
        ) {
            Row(
                modifier = Modifier.padding(20.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Icon(
                    Icons.Filled.Check,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.primary,
                    modifier = Modifier.size(24.dp)
                )
                Text(
                    "Lampcord is ready. Sign in or pick an account to start chatting.",
                    style = MaterialTheme.typography.bodyLarge,
                    modifier = Modifier.weight(1f)
                )
            }
        }
    }
}

@Composable
private fun OnboardingSwitchRow(
    title: String,
    subtitle: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Surface(
        onClick = { onCheckedChange(!checked) },
        shape = RoundedCornerShape(20.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Medium)
                Spacer(Modifier.height(2.dp))
                Text(
                    subtitle,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
            Spacer(Modifier.width(12.dp))
            ExpressiveSwitch(checked = checked, onCheckedChange = onCheckedChange)
        }
    }
}
