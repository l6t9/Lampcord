package me.lampu.lampcord.shared.ui.components.guilds

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.material3.SheetValue
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.model.Invite
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MemberListStore
import me.lampu.lampcord.shared.state.NavigationStore
import me.lampu.lampcord.shared.ui.components.DiscordBottomSheet
import me.lampu.lampcord.shared.ui.components.AsyncImage
import me.lampu.lampcord.shared.ui.components.ContainedLoadingIndicator
import me.lampu.lampcord.shared.ui.components.ShimmerBox
import me.lampu.lampcord.shared.ui.icons.Icons
import kotlinx.serialization.json.*
import kotlinx.coroutines.launch
import org.koin.compose.koinInject
import me.lampu.lampcord.shared.ui.components.rememberDiscordSheetState

@Composable
fun GuildProfileSkeleton() {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 32.dp)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(120.dp)
                .background(MaterialTheme.colorScheme.surfaceContainerHighest)
        )

        Box(modifier = Modifier.fillMaxWidth()) {
            Surface(
                modifier = Modifier
                    .padding(start = 16.dp)
                    .offset(y = (-40).dp)
                    .size(80.dp),
                shape = RoundedCornerShape(24.dp),
                color = MaterialTheme.colorScheme.surfaceContainerLow,
                border = androidx.compose.foundation.BorderStroke(4.dp, MaterialTheme.colorScheme.surfaceContainerLow)
            ) {
                ShimmerBox(modifier = Modifier.fillMaxSize(), shape = RoundedCornerShape(24.dp))
            }
        }

        Column(modifier = Modifier.padding(horizontal = 16.dp).offset(y = (-32).dp)) {
            ShimmerBox(modifier = Modifier.width(150.dp).height(24.dp), shape = RoundedCornerShape(12.dp))
            Spacer(Modifier.height(8.dp))
            ShimmerBox(modifier = Modifier.fillMaxWidth().height(40.dp), shape = RoundedCornerShape(8.dp))
            
            Spacer(Modifier.height(16.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                ShimmerBox(modifier = Modifier.size(8.dp), shape = CircleShape)
                Spacer(Modifier.width(4.dp))
                ShimmerBox(modifier = Modifier.width(60.dp).height(12.dp), shape = RoundedCornerShape(6.dp))
                Spacer(Modifier.width(16.dp))
                ShimmerBox(modifier = Modifier.size(8.dp), shape = CircleShape)
                Spacer(Modifier.width(4.dp))
                ShimmerBox(modifier = Modifier.width(80.dp).height(12.dp), shape = RoundedCornerShape(6.dp))
            }

            Spacer(Modifier.height(24.dp))
            ShimmerBox(modifier = Modifier.fillMaxWidth().height(40.dp), shape = RoundedCornerShape(8.dp))
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GuildProfileSheet(
    guildId: String,
    onDismiss: () -> Unit,
    guildApi: GuildApi = koinInject(),
    guildStore: GuildStore = koinInject(),
    navigationStore: NavigationStore = koinInject(),
    memberListStore: MemberListStore = koinInject()
) {
    var inviteData by remember { mutableStateOf<Invite?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(guildId) {
        isLoading = true
        val guild = guildStore.guilds.value.find { it.id == guildId }
        if (guild != null) {
            inviteData = Invite(code = "", guild = guild, approximate_member_count = guild.member_count)
            isLoading = false
        } else {
            val fetched = guildApi.getGuildPreview(guildId)
            if (fetched != null) {
                inviteData = Invite(
                    code = "", 
                    guild = fetched, 
                    approximate_member_count = fetched.approximate_member_count,
                    approximate_presence_count = fetched.approximate_presence_count
                )
            } else {
                val widget = guildApi.getGuildWidget(guildId)
                if (widget != null) {
                    val name = widget["name"]?.jsonPrimitive?.contentOrNull
                    val invite = widget["instant_invite"]?.jsonPrimitive?.contentOrNull
                    val presenceCount = widget["presence_count"]?.jsonPrimitive?.intOrNull
                    inviteData = Invite(
                        code = invite ?: "",
                        guild = Guild(id = guildId, name = name),
                        approximate_presence_count = presenceCount
                    )
                } else {
                    error = "Could not load server profile."
                }
            }
            isLoading = false
        }
    }

    DiscordBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberDiscordSheetState()
    ) {
        if (isLoading) {
            GuildProfileSkeleton()
        } else if (error != null) {
            Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                Text(error!!, color = MaterialTheme.colorScheme.error)
            }
        } else if (inviteData?.guild != null) {
            val guild = inviteData!!.guild!!
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(bottom = 32.dp)
            ) {
                Box(modifier = Modifier.fillMaxWidth()) {
                    val bannerUrl = guild.banner?.let { "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600" }
                    if (bannerUrl != null) {
                        AsyncImage(
                            model = bannerUrl,
                            contentDescription = null,
                            contentScale = ContentScale.Crop,
                            modifier = Modifier.fillMaxWidth().height(120.dp)
                        )
                    } else {
                        Box(
                            modifier = Modifier.fillMaxWidth().height(80.dp).background(MaterialTheme.colorScheme.primaryContainer)
                        )
                    }

                    Surface(
                        modifier = Modifier.padding(start = 16.dp).align(Alignment.BottomStart).offset(y = 40.dp).size(80.dp),
                        shape = RoundedCornerShape(24.dp),
                        color = MaterialTheme.colorScheme.surfaceContainerLow,
                        border = androidx.compose.foundation.BorderStroke(4.dp, MaterialTheme.colorScheme.surfaceContainerLow)
                    ) {
                        val iconUrl = guild.icon?.let { "https://cdn.discordapp.com/icons/${guild.id}/$it.png?size=160" }
                        if (iconUrl != null) {
                            AsyncImage(model = iconUrl, contentDescription = null, modifier = Modifier.fillMaxSize())
                        } else {
                            Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.secondaryContainer), contentAlignment = Alignment.Center) {
                                Text(guild.name?.take(1) ?: "?", style = MaterialTheme.typography.headlineMedium)
                            }
                        }
                    }
                }

                Spacer(Modifier.height(48.dp))

                Column(modifier = Modifier.padding(horizontal = 16.dp)) {
                    Text(text = guild.name ?: "Server", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
                    
                    if (guild.description?.isNotBlank() == true) {
                        Text(
                            text = guild.description,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(top = 8.dp)
                        )
                    }

                    Spacer(Modifier.height(16.dp))

                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFF23A559), CircleShape))
                        Spacer(Modifier.width(4.dp))
                        val onlineCount = guild.approximate_presence_count ?: memberListStore.getOnlineCount(guild.id)
                        Text(
                            "$onlineCount Online",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(Modifier.width(16.dp))
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFFB5BAC1), CircleShape))
                        Spacer(Modifier.width(4.dp))
                        val memberCount = guild.approximate_member_count ?: guild.member_count ?: memberListStore.getMemberCount(guild.id)
                        Text(
                            "${memberCount.takeIf { it > 0 } ?: 0} Members",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    Spacer(Modifier.height(24.dp))

                    val isMember = guildStore.guilds.value.any { it.id == guild.id }
                    val scope = rememberCoroutineScope()

                    Button(
                        onClick = {
                            if (isMember) {
                                navigationStore.selectGuild(guild) { /* subscribe */ }
                                onDismiss()
                            } else {
                                inviteData?.code?.let { code ->
                                    if (code.isNotBlank()) {
                                        scope.launch {
                                            val joined = guildApi.joinGuild(code)
                                            if (joined != null) {
                                                guildStore.handleGuildCreate(joined, emptyList())
                                                navigationStore.selectGuild(joined) { }
                                                onDismiss()
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(8.dp),
                        enabled = isMember || (inviteData?.code?.isNotBlank() == true)
                    ) {
                        Text(if (isMember) "Joined" else if (inviteData?.code?.isNotBlank() == true) "Join Server" else "Invite Required")
                    }
                }
            }
        }
    }
}
