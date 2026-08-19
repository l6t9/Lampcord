package me.lampu.lampcord.shared.ui.settings

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.state.UserStore
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.settings.*
import me.lampu.lampcord.shared.ui.icons.Icons
import org.koin.compose.koinInject

@Composable
fun ProfilesSettings(
    onBack: () -> Unit,
    userStore: UserStore = koinInject()
) {
    val user by userStore.currentUser.collectAsState()
    val userVal = user ?: return
    
    SettingsSubScreen(
        title = "Profiles",
        onNavigateBack = onBack
    ) {
        ProfileSettingsContent(userVal = userVal)
    }
}

@Composable
fun ProfileSettingsContent(userVal: me.lampu.lampcord.shared.model.User) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Material3SettingsGroup(
            title = "User Profile",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Profile Preview") },
                    description = {
                        Column(
                            modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                            verticalArrangement = Arrangement.spacedBy(24.dp)
                        ) {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(160.dp)
                            ) {
                                val bannerUrl = userVal.banner?.let { "https://cdn.discordapp.com/banners/${userVal.id}/$it.png?size=600" }
                                if (bannerUrl != null) {
                                    AsyncImage(
                                        model = bannerUrl,
                                        contentDescription = null,
                                        modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(8.dp))
                                    )
                                } else {
                                    Box(
                                        modifier = Modifier
                                            .fillMaxSize()
                                            .background(
                                                userVal.accent_color?.let { Color(it or 0xFF000000.toInt()) }
                                                    ?: MaterialTheme.colorScheme.primaryContainer,
                                                RoundedCornerShape(8.dp)
                                            )
                                    )
                                }

                                Surface(
                                    modifier = Modifier
                                        .padding(start = 16.dp)
                                        .align(Alignment.BottomStart)
                                        .offset(y = 20.dp)
                                        .size(80.dp),
                                    shape = CircleShape,
                                    color = MaterialTheme.colorScheme.surface,
                                    border = BorderStroke(4.dp, MaterialTheme.colorScheme.surface)
                                ) {
                                    val avatarUrl = userVal.avatar?.let { "https://cdn.discordapp.com/avatars/${userVal.id}/$it.png?size=160" }
                                    if (avatarUrl != null) {
                                        AsyncImage(model = avatarUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                                    } else {
                                        Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.secondaryContainer))
                                    }
                                }
                            }

                            Spacer(Modifier.height(16.dp))

                            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                                var displayName by remember { mutableStateOf(userVal.global_name ?: "") }
                                var pronouns by remember { mutableStateOf(userVal.pronouns ?: "") }
                                var bio by remember { mutableStateOf(userVal.bio ?: "") }

                                OutlinedTextField(
                                    value = displayName,
                                    onValueChange = { displayName = it },
                                    label = { Text("Display Name") },
                                    modifier = Modifier.fillMaxWidth()
                                )

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

                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.End
                                ) {
                                    Button(onClick = { /* TODO */ }) {
                                        Text("Save Changes")
                                    }
                                }
                            }
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Avatar",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Manage Avatar") },
                    description = {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                            horizontalArrangement = Arrangement.spacedBy(16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Button(onClick = { /* TODO */ }) {
                                Text("Change Avatar")
                            }
                            TextButton(onClick = { /* TODO */ }) {
                                Text("Remove Avatar", color = MaterialTheme.colorScheme.error)
                            }
                        }
                    }
                )
            )
        )

        Material3SettingsGroup(
            title = "Profile Themes",
            items = listOf(
                Material3SettingsItem(
                    title = { Text("Colors") },
                    description = {
                        Column(verticalArrangement = Arrangement.spacedBy(16.dp), modifier = Modifier.padding(top = 8.dp)) {
                            Text("Select colors for your profile banner and background.", style = MaterialTheme.typography.bodyMedium)
                            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                                Box(Modifier.size(40.dp).background(Color.Black, CircleShape))
                                Box(Modifier.size(40.dp).background(Color.Gray, CircleShape))
                            }
                        }
                    }
                )
            )
        )
    }
}
