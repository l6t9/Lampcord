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
import me.lampu.lampcord.shared.api.CdnUrls
import me.lampu.lampcord.shared.api.GuildApi
import me.lampu.lampcord.shared.api.JoinGuildResult
import me.lampu.lampcord.shared.generated.resources.Res
import me.lampu.lampcord.shared.generated.resources.guild_profile_join_server_button
import me.lampu.lampcord.shared.generated.resources.invite_embed_joined
import me.lampu.lampcord.shared.generated.resources.lampcord_guild_profile_invite_required
import me.lampu.lampcord.shared.generated.resources.lampcord_guild_profile_joining
import me.lampu.lampcord.shared.generated.resources.lampcord_guild_profile_load_failed
import me.lampu.lampcord.shared.generated.resources.lampcord_guild_profile_unnamed
import me.lampu.lampcord.shared.generated.resources.lampcord_invite_members_fmt
import me.lampu.lampcord.shared.generated.resources.lampcord_invite_online_fmt
import org.jetbrains.compose.resources.stringResource
import me.lampu.lampcord.shared.model.Invite
import me.lampu.lampcord.shared.model.Guild
import me.lampu.lampcord.shared.state.rememberGuildMediaUrls
import me.lampu.lampcord.shared.state.ChannelNavigator
import me.lampu.lampcord.shared.state.GuildStore
import me.lampu.lampcord.shared.state.MemberListStore
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
    invite: Invite? = null,
    guildApi: GuildApi = koinInject(),
    guildStore: GuildStore = koinInject(),
    channelNavigator: ChannelNavigator = koinInject(),
    memberListStore: MemberListStore = koinInject()
) {
    var inviteData by remember { mutableStateOf<Invite?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    var loadFailed by remember { mutableStateOf(false) }

    LaunchedEffect(guildId) {
        isLoading = true
        val local = profileInviteFor(guildStore.guilds.value.find { it.id == guildId }, invite)
        if (local != null) {
            inviteData = local
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
                    val widgetInvite = widget["instant_invite"]?.jsonPrimitive?.contentOrNull
                    val presenceCount = widget["presence_count"]?.jsonPrimitive?.intOrNull
                    inviteData = Invite(
                        code = widgetInvite ?: "",
                        guild = Guild(id = guildId, name = name),
                        approximate_presence_count = presenceCount
                    )
                } else {
                    loadFailed = true
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
        } else if (loadFailed) {
            Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                Text(stringResource(Res.string.lampcord_guild_profile_load_failed), color = MaterialTheme.colorScheme.error)
            }
        } else if (inviteData?.guild != null) {
            val profile = inviteData!!
            val guild = profile.guild!!
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(bottom = 32.dp)
            ) {
                Box(modifier = Modifier.fillMaxWidth()) {
                    val guildMedia = rememberGuildMediaUrls(
                        guildId = guild.id,
                        guildIconUrl = guild.icon?.let { CdnUrls.getGuildIconUrl(guild.id, guild.icon, 160) },
                        guildBannerUrl = guild.banner?.let { "https://cdn.discordapp.com/banners/${guild.id}/$it.png?size=600" }
                    )
                    val bannerUrl = guildMedia.banner
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
                        val iconUrl = guildMedia.icon
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
                    Text(text = guild.name ?: stringResource(Res.string.lampcord_guild_profile_unnamed), style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)

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
                        val onlineCount = profile.approximate_presence_count
                            ?: guild.approximate_presence_count
                            ?: memberListStore.getOnlineCount(guild.id)
                        Text(
                            stringResource(Res.string.lampcord_invite_online_fmt, onlineCount),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(Modifier.width(16.dp))
                        Box(modifier = Modifier.size(8.dp).background(Color(0xFFB5BAC1), CircleShape))
                        Spacer(Modifier.width(4.dp))
                        val memberCount = profile.approximate_member_count
                            ?: guild.approximate_member_count
                            ?: guild.member_count
                            ?: memberListStore.getMemberCount(guild.id)
                        Text(
                            stringResource(Res.string.lampcord_invite_members_fmt, memberCount.coerceAtLeast(0)),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    Spacer(Modifier.height(24.dp))

                    val isMember = guildStore.guilds.value.any { it.id == guild.id }
                    val scope = rememberCoroutineScope()
                    var joinError by remember { mutableStateOf<JoinGuildResult.Failed?>(null) }
                    var joining by remember { mutableStateOf(false) }
                    val joinCode = joinCodeFor(invite?.code, inviteData?.code)

                    Button(
                        onClick = {
                            if (isMember) {
                                channelNavigator.navigateToGuild(guild.id)
                                onDismiss()
                            } else {
                                joinCode?.takeIf { !joining }?.let { code ->
                                    scope.launch {
                                        joining = true
                                        joinError = null
                                        try {
                                            when (val result = guildApi.joinGuild(code)) {
                                                is JoinGuildResult.Joined -> {
                                                    guildStore.joinCompleted(result.guild)
                                                    channelNavigator.navigateToGuild(result.guild.id)
                                                    onDismiss()
                                                }
                                                is JoinGuildResult.Failed -> joinError = result
                                            }
                                        } finally {
                                            joining = false
                                        }
                                    }
                                }
                            }
                        },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(8.dp),
                        enabled = isMember || (joinCode != null && !joining)
                    ) {
                        Text(
                            when {
                                isMember -> stringResource(Res.string.invite_embed_joined)
                                joining -> stringResource(Res.string.lampcord_guild_profile_joining)
                                joinCode != null -> stringResource(Res.string.guild_profile_join_server_button)
                                else -> stringResource(Res.string.lampcord_guild_profile_invite_required)
                            }
                        )
                    }

                    joinError?.let {
                        Text(
                            joinFailureText(it),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.error,
                            modifier = Modifier.padding(top = 8.dp)
                        )
                    }
                }
            }
        }
    }
}

internal fun joinCodeFor(explicit: String?, fetched: String?): String? =
    explicit?.trim()?.takeIf { it.isNotEmpty() }
        ?: fetched?.trim()?.takeIf { it.isNotEmpty() }

internal fun profileInviteFor(storeGuild: Guild?, invite: Invite?): Invite? = when {
    storeGuild != null -> Invite(
        code = invite?.code.orEmpty(),
        guild = storeGuild,
        approximate_member_count = storeGuild.member_count ?: invite?.approximate_member_count,
        approximate_presence_count = invite?.approximate_presence_count,
    )
    invite?.guild != null -> invite
    else -> null
}
