package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.HsvColorPicker
import me.lampu.lampcord.shared.ui.components.profiles.ProfileCard
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.utils.showToast
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.model.UserProfile
import me.lampu.lampcord.shared.model.UserProfileMetadata

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ProfilesSettings(
    onBack: () -> Unit,
    userStore: UserStore = koinInject(),
    userApi: me.lampu.lampcord.shared.api.UserApi = koinInject()
) {
    val user by userStore.currentUser.collectAsState()
    val userVal = user ?: return
    
    SettingsSubScreen(
        title = "Profiles",
        onNavigateBack = onBack
    ) {
        ProfileSettingsContent(userVal = userVal, userApi = userApi, userStore = userStore)
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ProfileSettingsContent(
    userVal: me.lampu.lampcord.shared.model.User,
    userApi: me.lampu.lampcord.shared.api.UserApi = koinInject(),
    userStore: UserStore = koinInject(),
    guildStore: me.lampu.lampcord.shared.state.GuildStore = koinInject(),
    guildApi: me.lampu.lampcord.shared.api.GuildApi = koinInject(),
    clientProfileStore: me.lampu.lampcord.shared.state.ClientProfileStore = koinInject(),
    settingsStore: me.lampu.lampcord.shared.state.SettingsStore = koinInject()
) {
    val scope = rememberCoroutineScope()
    val guilds by guildStore.guilds.collectAsState()
    
    var selectedGuildId by remember { mutableStateOf<String?>(null) }
    val selectedGuild = remember(selectedGuildId, guilds) { guilds.find { it.id == selectedGuildId } }
    
    var serverProfile by remember { mutableStateOf<UserProfile?>(null) }
    var isLoadingProfile by remember { mutableStateOf(false) }

    val members by userStore.members.collectAsState()
    val currentMember = remember(selectedGuildId, members) {
        selectedGuildId?.let { userStore.getMember(it, userVal.id) }
    }
    val uriHandler = androidx.compose.ui.platform.LocalUriHandler.current
    
    val customProfiles by clientProfileStore.customProfiles.collectAsState()
    val localOverrides by clientProfileStore.localOverrides.collectAsState()
    val customProfile = remember(userVal.id, customProfiles, localOverrides) {
        clientProfileStore.getCustomProfile(userVal.id)
    }
    
    var displayName by remember(userVal.id, selectedGuildId) { 
        mutableStateOf("") 
    }
    var pronouns by remember(userVal.id, selectedGuildId) { 
        mutableStateOf("") 
    }
    var bio by remember(userVal.id, selectedGuildId) { 
        mutableStateOf("") 
    }
    var bannerColor by remember(userVal.id, selectedGuildId) {
        mutableStateOf<Int?>(null) 
    }
    var bannerUri by remember(userVal.id, selectedGuildId) {
        mutableStateOf<String?>(null)
    }
    var themePrimaryColor by remember(userVal.id, selectedGuildId) {
        mutableStateOf<Int?>(null)
    }
    var themeSecondaryColor by remember(userVal.id, selectedGuildId) {
        mutableStateOf<Int?>(null)
    }
    
    var showColorPicker by remember { mutableStateOf(false) }
    var colorPickerTarget by remember { mutableStateOf(0) } // 0: banner, 1: primary, 2: secondary
    var showAvatarPicker by remember { mutableStateOf(false) }
    var showBannerPicker by remember { mutableStateOf(false) }
    var showGuildPicker by remember { mutableStateOf(false) }

    val hasNitro = (userVal.premium_type ?: 0) > 0

    val previewCustomProfile = remember(customProfile, bannerUri, themePrimaryColor, themeSecondaryColor, bannerColor) {
        (customProfile ?: me.lampu.lampcord.shared.model.CustomProfile(user_id = userVal.id)).copy(
            banner = bannerUri,
            theme_colors = listOfNotNull(themePrimaryColor, themeSecondaryColor).ifEmpty { null },
            accent_color = bannerColor
        )
    }

    val previewProfile = remember(userVal, displayName, pronouns, bio, bannerColor, themePrimaryColor, themeSecondaryColor, selectedGuildId, serverProfile) {
        val themeColors = listOfNotNull(themePrimaryColor, themeSecondaryColor).ifEmpty { null }
        val metadata = UserProfileMetadata(
            bio = bio,
            accent_color = bannerColor,
            pronouns = pronouns,
            theme_colors = themeColors
        )
        
        serverProfile?.copy(
            user = userVal.copy(
                global_name = if (selectedGuildId == null) displayName else userVal.global_name,
                pronouns = if (selectedGuildId == null) pronouns else userVal.pronouns,
                bio = if (selectedGuildId == null) bio else userVal.bio,
                accent_color = if (selectedGuildId == null) bannerColor else userVal.accent_color
            ),
            user_profile = if (selectedGuildId == null) metadata else serverProfile?.user_profile,
            guild_member_profile = if (selectedGuildId != null) metadata else serverProfile?.guild_member_profile,
            guild_member = if (selectedGuildId != null) serverProfile?.guild_member?.copy(nick = displayName) else serverProfile?.guild_member,
            guild_id = selectedGuildId
        ) ?: UserProfile(
            user = userVal.copy(
                global_name = if (selectedGuildId == null) displayName else userVal.global_name,
                accent_color = if (selectedGuildId == null) bannerColor else userVal.accent_color,
                pronouns = if (selectedGuildId == null) pronouns else userVal.pronouns,
                bio = if (selectedGuildId == null) bio else userVal.bio
            ),
            user_profile = if (selectedGuildId == null) metadata else null,
            guild_member_profile = if (selectedGuildId != null) metadata else null,
            guild_id = selectedGuildId
        )
    }

    LaunchedEffect(selectedGuildId, customProfile, userVal, currentMember) {
        // Initial set from stores/userVal
        if (selectedGuildId == null) {
            displayName = userVal.global_name ?: ""
            bio = userVal.bio?.let { me.lampu.lampcord.shared.ui.components.profiles.Profile3y3.strip(it) } ?: ""
            pronouns = userVal.pronouns ?: ""
            bannerColor = customProfile?.accent_color ?: userVal.accent_color
        } else {
            displayName = currentMember?.nick ?: ""
            bannerColor = customProfile?.accent_color
        }
        bannerUri = customProfile?.banner
        themePrimaryColor = customProfile?.theme_colors?.getOrNull(0)
        themeSecondaryColor = customProfile?.theme_colors?.getOrNull(1)

        isLoadingProfile = true
        val profile = if (selectedGuildId != null) {
            userApi.getUserProfile(userVal.id, selectedGuildId)
        } else {
            userApi.getUserProfile(userVal.id)
        }
        isLoadingProfile = false
        serverProfile = profile
        
        if (selectedGuildId != null) {
            profile?.guild_member?.let {
                userStore.cacheMember(selectedGuildId!!, userVal.id, it)
                displayName = it.nick ?: ""
            }
            profile?.guild_member_profile?.let {
                bio = it.bio?.let { b -> me.lampu.lampcord.shared.ui.components.profiles.Profile3y3.strip(b) } ?: ""
                pronouns = it.pronouns ?: ""
                // Use customProfile if set, otherwise fallback to server values
                bannerColor = customProfile?.accent_color ?: it.accent_color
                bannerUri = customProfile?.banner
                themePrimaryColor = it.theme_colors?.getOrNull(0) ?: customProfile?.theme_colors?.getOrNull(0)
                themeSecondaryColor = it.theme_colors?.getOrNull(1) ?: customProfile?.theme_colors?.getOrNull(1)
            }
        } else {
            displayName = userVal.global_name ?: ""
            bio = userVal.bio?.let { b -> me.lampu.lampcord.shared.ui.components.profiles.Profile3y3.strip(b) } ?: ""
            pronouns = userVal.pronouns ?: ""
            bannerColor = customProfile?.accent_color ?: userVal.accent_color
            bannerUri = customProfile?.banner
            themePrimaryColor = profile?.user_profile?.theme_colors?.getOrNull(0) ?: customProfile?.theme_colors?.getOrNull(0)
            themeSecondaryColor = profile?.user_profile?.theme_colors?.getOrNull(1) ?: customProfile?.theme_colors?.getOrNull(1)
        }
    }



    if (showGuildPicker) {
        AlertDialog(
            onDismissRequest = { showGuildPicker = false },
            title = { Text("Select Server") },
            text = {
                LazyColumn(modifier = Modifier.fillMaxWidth().heightIn(max = 400.dp)) {
                    item {
                        ListItem(
                            headlineContent = { Text("Global Profile") },
                            modifier = Modifier.clickable { 
                                selectedGuildId = null
                                showGuildPicker = false
                            },
                            leadingContent = {
                                Icon(Icons.Rounded.Public, null)
                            }
                        )
                    }
                    items(guilds) { guild ->
                        ListItem(
                            headlineContent = { Text(guild.name ?: "Unknown") },
                            modifier = Modifier.clickable {
                                selectedGuildId = guild.id
                                showGuildPicker = false
                            },
                            leadingContent = {
                                val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=64" }
                                if (iconUrl != null) {
                                    AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.size(32.dp).clip(CircleShape))
                                } else {
                                    Box(Modifier.size(32.dp).background(MaterialTheme.colorScheme.secondaryContainer, CircleShape), contentAlignment = Alignment.Center) {
                                        Text(guild.name?.take(1) ?: "?", style = MaterialTheme.typography.labelSmall)
                                    }
                                }
                            }
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { showGuildPicker = false }) { Text("Cancel") }
            }
        )
    }

    if (showColorPicker) {
        AlertDialog(
            onDismissRequest = { showColorPicker = false },
            confirmButton = {},
            text = {
                val initialColor = when (colorPickerTarget) {
                    0 -> bannerColor
                    1 -> themePrimaryColor
                    else -> themeSecondaryColor
                }
                HsvColorPicker(
                    initialColor = initialColor?.let { Color(it or 0xFF000000.toInt()) } ?: Color.Gray,
                    onColorSelected = { selectedColor ->
                        val colorInt = (selectedColor.red * 255).toInt() shl 16 or
                                     ((selectedColor.green * 255).toInt() shl 8) or
                                     (selectedColor.blue * 255).toInt()
                        when (colorPickerTarget) {
                            0 -> bannerColor = colorInt
                            1 -> themePrimaryColor = colorInt
                            2 -> themeSecondaryColor = colorInt
                        }
                        showColorPicker = false
                    },
                    onDismiss = { showColorPicker = false }
                )
            },
            properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)
        )
    }

    me.lampu.lampcord.shared.utils.FilePicker(
        show = showAvatarPicker,
        onDismiss = { showAvatarPicker = false },
        onFileSelected = { files ->
            val file = files.firstOrNull() ?: return@FilePicker
            val fileName = file.first.lowercase()
            val mimeType = when {
                fileName.endsWith(".png") -> "image/png"
                fileName.endsWith(".jpg") || fileName.endsWith(".jpeg") -> "image/jpeg"
                fileName.endsWith(".gif") -> "image/gif"
                fileName.endsWith(".webp") -> "image/webp"
                else -> "image/png"
            }
            val base64 = me.lampu.lampcord.shared.utils.base64Encode(file.second)
            val dataUri = "data:$mimeType;base64,$base64"
            
            scope.launch {
                if (selectedGuildId == null) {
                    val updated = userApi.patchUser(me.lampu.lampcord.shared.model.User.Partial(avatar = dataUri))
                    if (updated != null) {
                        userStore.handleUserUpdate(updated)
                    }
                } else {
                    val updated = guildApi.updateSelfMember(selectedGuildId!!, me.lampu.lampcord.shared.model.Member.Partial(avatar = dataUri))
                    if (updated != null) {
                        userStore.cacheMember(selectedGuildId!!, userVal.id, updated)
                    }
                }
            }
            showAvatarPicker = false
        }
    )

    me.lampu.lampcord.shared.utils.FilePicker(
        show = showBannerPicker,
        onDismiss = { showBannerPicker = false },
        onFileSelected = { files ->
            val file = files.firstOrNull() ?: return@FilePicker
            val fileName = file.first.lowercase()
            val mimeType = when {
                fileName.endsWith(".png") -> "image/png"
                fileName.endsWith(".jpg") || fileName.endsWith(".jpeg") -> "image/jpeg"
                fileName.endsWith(".gif") -> "image/gif"
                fileName.endsWith(".webp") -> "image/webp"
                else -> "image/png"
            }
            val base64 = me.lampu.lampcord.shared.utils.base64Encode(file.second)
            val dataUri = "data:$mimeType;base64,$base64"
            
            bannerUri = dataUri
            showBannerPicker = false
        }
    )

    Column(modifier = Modifier.fillMaxWidth()) {
        Surface(
            onClick = { showGuildPicker = true },
            modifier = Modifier.fillMaxWidth().padding(16.dp),
            shape = RoundedCornerShape(12.dp),
            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
        ) {
            Row(
                modifier = Modifier.padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                if (selectedGuildId == null) {
                    Icon(Icons.Rounded.Public, null, modifier = Modifier.size(32.dp))
                } else {
                    val iconUrl = selectedGuild?.icon?.let { "https://cdn.discordapp.com/icons/${selectedGuild.id}/$it.png?size=64" }
                    if (iconUrl != null) {
                        AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.size(32.dp).clip(CircleShape))
                    } else {
                        Box(Modifier.size(32.dp).background(MaterialTheme.colorScheme.secondaryContainer, CircleShape), contentAlignment = Alignment.Center) {
                            Text(selectedGuild?.name?.take(1) ?: "?", style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
                Spacer(Modifier.width(16.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text("Editing Profile for:", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text(selectedGuild?.name ?: "Global Profile", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                }
                Icon(Icons.Rounded.ExpandMore, null)
            }
        }

        if (isLoadingProfile) {
            Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(24.dp)
            ) {
                // LEFT SIDE: PREVIEW
                Column(
                    modifier = Modifier.width(300.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    ProfileCard(
                        profile = previewProfile,
                        isExpanded = false,
                        onEditBanner = { showBannerPicker = true },
                        onEditAvatar = { showAvatarPicker = true },
                        customProfileOverride = previewCustomProfile,
                        modifier = Modifier.fillMaxWidth().heightIn(max = 600.dp)
                    )

                    // Action Buttons: Remove Avatar/Banner and Save
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        ButtonGroup(
                            modifier = Modifier.fillMaxWidth().height(48.dp),
                            overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
                            horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
                        ) {
                            customItem(
                                buttonGroupContent = {
                                    FilledTonalButton(
                                        onClick = {
                                            scope.launch {
                                                if (selectedGuildId == null) {
                                                    val updated = userApi.patchUser(me.lampu.lampcord.shared.model.User.Partial(avatar = null))
                                                    if (updated != null) userStore.handleUserUpdate(updated)
                                                } else {
                                                    val updated = guildApi.updateSelfMember(selectedGuildId!!, me.lampu.lampcord.shared.model.Member.Partial(avatar = null))
                                                    if (updated != null) userStore.cacheMember(selectedGuildId!!, userVal.id, updated)
                                                }
                                            }
                                        },
                                        shapes = ButtonDefaults.shapes(
                                            shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                                            pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape,
                                        ),
                                        modifier = Modifier.weight(1f).fillMaxHeight(),
                                    ) {
                                        Icon(Icons.Rounded.Delete, null, modifier = Modifier.size(18.dp))
                                        Spacer(Modifier.width(8.dp))
                                        Text("No Avatar", maxLines = 1)
                                    }
                                },
                                menuContent = { menuState ->
                                    DropdownMenuItem(
                                        text = { Text("No Avatar") },
                                        onClick = {
                                            scope.launch {
                                                if (selectedGuildId == null) {
                                                    val updated = userApi.patchUser(me.lampu.lampcord.shared.model.User.Partial(avatar = null))
                                                    if (updated != null) userStore.handleUserUpdate(updated)
                                                } else {
                                                    val updated = guildApi.updateSelfMember(selectedGuildId!!, me.lampu.lampcord.shared.model.Member.Partial(avatar = null))
                                                    if (updated != null) userStore.cacheMember(selectedGuildId!!, userVal.id, updated)
                                                }
                                            }
                                            menuState.dismiss()
                                        },
                                        leadingIcon = { Icon(Icons.Rounded.Delete, null) }
                                    )
                                }
                            )

                            customItem(
                                buttonGroupContent = {
                                    FilledTonalButton(
                                        onClick = { bannerUri = "" },
                                        shapes = ButtonDefaults.shapes(
                                            shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                                            pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape,
                                        ),
                                        modifier = Modifier.weight(1f).fillMaxHeight(),
                                    ) {
                                        Icon(Icons.Rounded.Delete, null, modifier = Modifier.size(18.dp))
                                        Spacer(Modifier.width(8.dp))
                                        Text("No Banner", maxLines = 1)
                                    }
                                },
                                menuContent = { menuState ->
                                    DropdownMenuItem(
                                        text = { Text("No Banner") },
                                        onClick = {
                                            bannerUri = ""
                                            menuState.dismiss()
                                        },
                                        leadingIcon = { Icon(Icons.Rounded.Delete, null) }
                                    )
                                }
                            )
                        }

                        Button(
                            onClick = {
                                scope.launch {
                                    val themeColors = listOfNotNull(themePrimaryColor, themeSecondaryColor)
                                    
                                    var finalBio = bio
                                    if (!hasNitro && settingsStore.profile3y3) {
                                        val custom = me.lampu.lampcord.shared.model.CustomProfile(
                                            theme_colors = themeColors.ifEmpty { null },
                                            accent_color = bannerColor,
                                            banner = bannerUri
                                        )
                                        val encoded = me.lampu.lampcord.shared.ui.components.profiles.Profile3y3.encode(
                                            kotlinx.serialization.json.Json.encodeToString(custom)
                                        )
                                        finalBio += encoded
                                    }

                                    if (selectedGuildId == null) {
                                        val updated = userApi.patchUser(
                                            me.lampu.lampcord.shared.model.User.Partial(
                                                global_name = displayName,
                                                pronouns = pronouns,
                                                bio = finalBio,
                                                accent_color = bannerColor,
                                                banner = if (hasNitro) bannerUri else null
                                            )
                                        )
                                        if (updated != null) {
                                            userStore.handleUserUpdate(updated)
                                        }

                                        if (hasNitro) {
                                            userApi.patchUserProfile(
                                                me.lampu.lampcord.shared.model.UserProfileMetadata.Partial(
                                                    theme_colors = themeColors.ifEmpty { null },
                                                    accent_color = bannerColor,
                                                    banner = bannerUri
                                                )
                                            )
                                        } else {
                                            val custom = me.lampu.lampcord.shared.model.CustomProfile(
                                                user_id = userVal.id,
                                                theme_colors = themeColors.ifEmpty { null },
                                                accent_color = bannerColor,
                                                banner = bannerUri
                                            )
                                            clientProfileStore.setLocalOverride(userVal.id, custom)
                                        }
                                    } else {
                                        val updated = guildApi.updateSelfMember(
                                            selectedGuildId!!,
                                            me.lampu.lampcord.shared.model.Member.Partial(
                                                nick = displayName,
                                                banner = if (hasNitro) bannerUri else null
                                            )
                                        )
                                        if (updated != null) {
                                            userStore.cacheMember(selectedGuildId!!, userVal.id, updated)
                                        }
                                        
                                        // Update server profile metadata if we have nitro
                                        if (hasNitro) {
                                            // Usually Discord has a different endpoint for guild profile metadata
                                            // but for now we focus on global consistency.
                                        } else {
                                            val custom = me.lampu.lampcord.shared.model.CustomProfile(
                                                user_id = userVal.id,
                                                theme_colors = themeColors.ifEmpty { null },
                                                accent_color = bannerColor,
                                                banner = bannerUri
                                            )
                                            clientProfileStore.setLocalOverride(userVal.id, custom)
                                        }
                                    }
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                            shape = CircleShape,
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = MaterialTheme.colorScheme.onPrimary
                            )
                        ) {
                            Text("Save Changes")
                        }
                    }
                }

                // RIGHT SIDE: SETTINGS
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    Material3SettingsGroup(
                        title = if (selectedGuildId == null) "User Profile" else "Server Profile",
                        items = listOf(
                            Material3SettingsItem(
                                title = { Text("Details") },
                                description = {
                                    Column(modifier = Modifier.fillMaxWidth().padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                                        OutlinedTextField(
                                            value = displayName,
                                            onValueChange = { displayName = it },
                                            label = { Text(if (selectedGuildId == null) "Display Name" else "Server Nickname") },
                                            modifier = Modifier.fillMaxWidth()
                                        )

                                        if (selectedGuildId == null) {
                                            OutlinedTextField(
                                                value = pronouns,
                                                onValueChange = { pronouns = it },
                                                label = { Text("Pronouns") },
                                                modifier = Modifier.fillMaxWidth()
                                            )

                                            OutlinedTextField(
                                                value = bio,
                                                onValueChange = { bio = it },
                                                label = { Text("About Me") },
                                                modifier = Modifier.fillMaxWidth(),
                                                minLines = 3
                                            )
                                        }
                                    }
                                }
                            )
                        )
                    )

                    Material3SettingsGroup(
                        title = "Banner Color",
                        items = listOf(
                            Material3SettingsItem(
                                title = { Text("Customize Color") },
                                description = {
                                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.padding(top = 8.dp)) {
                                        Text("Used when no banner image is set:", style = MaterialTheme.typography.bodyMedium)
                                        Box(
                                            modifier = Modifier
                                                .size(32.dp)
                                                .background(bannerColor?.let { Color(it or 0xFF000000.toInt()) } ?: Color.Gray, CircleShape)
                                                .clickable { 
                                                    colorPickerTarget = 0
                                                    showColorPicker = true 
                                                }
                                        )
                                    }
                                }
                            )
                        )
                    )

                    Material3SettingsGroup(
                        title = "Profile Themes",
                        items = listOf(
                            Material3SettingsItem(
                                title = { Text("Theme Colors") },
                                description = {
                                    Column(modifier = Modifier.fillMaxWidth().padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                                        if (hasNitro) {
                                            Text("Set nitro profile colors. These will be visible to everyone.", style = MaterialTheme.typography.bodySmall)
                                        } else {
                                            val localNotice = if (settingsStore.profile3y3) 
                                                "Profile colors are shared via invisible text in your bio since you don't have Nitro."
                                                else "Nitro is required for global profile colors. These colors will only be visible to you locally."
                                            Text(localNotice, style = MaterialTheme.typography.bodySmall, color = if (settingsStore.profile3y3) Color.Unspecified else MaterialTheme.colorScheme.onSurfaceVariant)
                                        }
                                        
                                        Row(modifier = Modifier.padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                                Box(
                                                    modifier = Modifier
                                                        .size(40.dp)
                                                        .background(themePrimaryColor?.let { Color(it or 0xFF000000.toInt()) } ?: Color.Gray, CircleShape)
                                                        .clickable { 
                                                            colorPickerTarget = 1
                                                            showColorPicker = true 
                                                        }
                                                )
                                                Text("Primary", style = MaterialTheme.typography.labelSmall)
                                            }
                                            
                                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                                Box(
                                                    modifier = Modifier
                                                        .size(40.dp)
                                                        .background(themeSecondaryColor?.let { Color(it or 0xFF000000.toInt()) } ?: Color.Gray, CircleShape)
                                                        .clickable { 
                                                            colorPickerTarget = 2
                                                            showColorPicker = true 
                                                        }
                                                )
                                                Text("Secondary", style = MaterialTheme.typography.labelSmall)
                                            }
                                        }
                                    }
                                }
                            )
                        )
                    )
                }
            }

    var show3y3Confirmation by remember { mutableStateOf(false) }
    var showUserBgConfirmation by remember { mutableStateOf(false) }
    var showUserPfpConfirmation by remember { mutableStateOf(false) }

    if (show3y3Confirmation) {
        AlertDialog(
            onDismissRequest = { show3y3Confirmation = false },
            title = { Text("Enable 3y3 Profile Colors?") },
            text = { Text("By enabling this, your profile customizations will be embedded into your profile's bio using invisible text. This allows other Lampcord users to see your custom theme and banner without needing Nitro.") },
            confirmButton = {
                Button(onClick = {
                    settingsStore.profile3y3 = true
                    show3y3Confirmation = false
                }) {
                    Text("Enable")
                }
            },
            dismissButton = {
                TextButton(onClick = { show3y3Confirmation = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    if (showUserBgConfirmation) {
        AlertDialog(
            onDismissRequest = { showUserBgConfirmation = false },
            title = { Text("Enable UserBG Banners?") },
            text = { Text("By enabling this, you will see custom banners from the UserBG database. This feature relies on external community-maintained data.") },
            confirmButton = {
                Button(onClick = {
                    settingsStore.userBg = true
                    showUserBgConfirmation = false
                }) {
                    Text("Enable")
                }
            },
            dismissButton = {
                TextButton(onClick = { showUserBgConfirmation = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    if (showUserPfpConfirmation) {
        AlertDialog(
            onDismissRequest = { showUserPfpConfirmation = false },
            title = { Text("Enable UserPFP Icons?") },
            text = { Text("By enabling this, you will see custom profile icons from the UserPFP database. This feature relies on external community-maintained data.") },
            confirmButton = {
                Button(onClick = {
                    settingsStore.userPfp = true
                    showUserPfpConfirmation = false
                }) {
                    Text("Enable")
                }
            },
            dismissButton = {
                TextButton(onClick = { showUserPfpConfirmation = false }) {
                    Text("Cancel")
                }
            }
        )
    }

    Material3SettingsGroup(
        title = "Profile Enhancements",
        items = listOf(
            switchSettingsItem(
                title = "3y3 Profile Colors",
                description = "Share and see custom profile colors via invisible bio text.",
                checked = settingsStore.profile3y3,
                onCheckedChange = { checked ->
                    if (checked) {
                        show3y3Confirmation = true
                    } else {
                        settingsStore.profile3y3 = false
                    }
                }
            ),
            switchSettingsItem(
                title = "UserBG Banners",
                description = "See custom banners set via the UserBG database.",
                checked = settingsStore.userBg,
                onCheckedChange = { checked ->
                    if (checked) {
                        showUserBgConfirmation = true
                    } else {
                        settingsStore.userBg = false
                    }
                }
            ),
            switchSettingsItem(
                title = "UserPFP Icons",
                description = "See custom icons set via the UserPFP database.",
                checked = settingsStore.userPfp,
                onCheckedChange = { checked ->
                    if (checked) {
                        showUserPfpConfirmation = true
                    } else {
                        settingsStore.userPfp = false
                    }
                }
            ),
            Material3SettingsItem(
                icon = Icons.Rounded.Public,
                title = { Text("Set Custom Banner (Blackbox)") },
                description = { Text("Join the Blackbox server to set your custom banner for UserBG.") },
                onClick = {
                    uriHandler.openUri("https://discord.gg/S89Y99m")
                }
            ),
            Material3SettingsItem(
                icon = Icons.Rounded.MusicNote,
                title = { Text("Connect Last.fm") },
                description = { Text("Display your recent scrobbles and top tracks on your Lampcord profile.") },
                onClick = {
                    showToast("Last.fm integration coming soon!")
                }
            )
        )
    )
        }
    }
}
