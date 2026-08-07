package me.lampu.lampcord.shared.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.lampu.lampcord.shared.ui.icons.Icons
import me.lampu.lampcord.shared.ui.components.AsyncImage

enum class AboutContributorRole {
    CREATOR,
    MAIN_DEVELOPER,
    CONTRIBUTOR,
}

enum class AboutAvatarShape {
    COOKIE_4_SIDED,
    COOKIE_6_SIDED,
    COOKIE_9_SIDED,
    BURST,
    PUFFY_DIAMOND,
    SUNNY,
    GEM,
}

data class AboutContributor(
    val name: String,
    val role: AboutContributorRole,
    val githubHandle: String? = null,
    val websiteUrl: String? = null,
    val sponsorUrl: String? = null,
    val avatarShape: AboutAvatarShape,
) {
    val avatarUrl: String = "https://github.com/${githubHandle}.png"
    val githubUrl: String? = githubHandle?.let { "https://github.com/$it" }
}

data class AboutScreenText(
    val title: String = "About",
    val contributors: String = "Contributors",
    val communityAndInfo: String = "Community & Info",
    val openWebsite: String = "Open Website",
    val openGithubProfile: String = "Open GitHub Profile",
    val supportDevelopment: String = "Support the development",
) {
    fun roleLabel(role: AboutContributorRole): String = when (role) {
        AboutContributorRole.CREATOR -> "Creator"
        AboutContributorRole.MAIN_DEVELOPER -> "Main Developer"
        AboutContributorRole.CONTRIBUTOR -> "Contributor"
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun AboutContent(
    version: String,
    onOpenUrl: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val text = AboutScreenText()
    val leadDeveloper = AboutContributor(
        name = "Lamp",
        role = AboutContributorRole.MAIN_DEVELOPER,
        githubHandle = "l6t9",
        websiteUrl = "https://lamp.delivery",
        sponsorUrl = "https://github.com/sponsors/l6t9",
        avatarShape = AboutAvatarShape.COOKIE_9_SIDED
    )

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        // App Header Card
        AppHeaderCard(version)

        Spacer(Modifier.height(24.dp))

        // Lead Developer Card
        LeadDeveloperCard(
            contributor = leadDeveloper,
            text = text,
            onOpenUrl = onOpenUrl
        )

        Spacer(Modifier.height(32.dp))

        // Contributors Section (excluding lead if any others)
        val otherContributors = emptyList<AboutContributor>() // Add more here if needed
        if (otherContributors.isNotEmpty()) {
            ContributorsSection(
                contributors = otherContributors,
                text = text,
                onOpenUrl = onOpenUrl
            )
            Spacer(Modifier.height(32.dp))
        }

        // Links Section
        val links = listOf(
            Triple("Discord", "https://discord.gg/uHXJJzxSD8", Icons.Brand.Discord),
            Triple("GitHub Repository", "https://github.com/l6t9/lampcord", Icons.Brand.Github),
            Triple("License", "https://github.com/l6t9/lampcord/blob/main/LICENSE", Icons.Filled.Info)
        )
        
        AboutSection(
            title = text.communityAndInfo,
            itemCount = links.size,
            itemContent = { index ->
                val (name, _, icon) = links[index]
                Row(
                    modifier = Modifier.fillMaxWidth().padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(24.dp),
                    )
                    Spacer(Modifier.width(16.dp))
                    Text(text = name, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                }
            },
            onClick = { index -> onOpenUrl(links[index].second) }
        )
        
        Spacer(Modifier.height(32.dp))
    }
}

@Composable
private fun AppHeaderCard(version: String) {
    ElevatedCard(
        shape = RoundedCornerShape(32.dp),
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.elevatedCardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().padding(24.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(64.dp)
                    .background(MaterialTheme.colorScheme.primary, RoundedCornerShape(16.dp)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    Icons.Brand.Discord,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onPrimary,
                    modifier = Modifier.size(40.dp)
                )
            }
            Spacer(Modifier.width(20.dp))
            Column {
                Text(
                    text = "Materialcord",
                    style = MaterialTheme.typography.headlineLarge,
                    fontWeight = FontWeight.Black,
                    color = MaterialTheme.colorScheme.onSurface,
                    letterSpacing = (-0.5).sp,
                )
                Spacer(Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    InfoBadge(version, MaterialTheme.colorScheme.primary.copy(alpha = 0.1f), MaterialTheme.colorScheme.primary)
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun LeadDeveloperCard(
    contributor: AboutContributor,
    text: AboutScreenText,
    onOpenUrl: (String) -> Unit,
) {
    ElevatedCard(
        shape = RoundedCornerShape(32.dp),
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.elevatedCardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
    ) {
        Column(Modifier.padding(24.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(20.dp),
                modifier = Modifier.fillMaxWidth(),
            ) {
                ContributorAvatar(
                    contributor = contributor,
                    sizeDp = 110,
                )
                Column(verticalArrangement = Arrangement.Center) {
                    Text(
                        text = contributor.name,
                        style = MaterialTheme.typography.headlineLarge,
                        fontWeight = FontWeight.Black,
                        color = MaterialTheme.colorScheme.onSurface,
                        lineHeight = 38.sp,
                        letterSpacing = (-0.5).sp,
                    )
                    Text(
                        text = text.roleLabel(contributor.role),
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.primary,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
            Spacer(Modifier.height(24.dp))
            DeveloperSocials(contributor, text, onOpenUrl)
            Spacer(Modifier.height(16.dp))
            Button(
                onClick = { contributor.sponsorUrl?.let(onOpenUrl) },
                enabled = contributor.sponsorUrl != null,
                modifier = Modifier.fillMaxWidth(),
                shape = CircleShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.primary,
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                ),
            ) {
                Icon(Icons.Filled.Favorite, contentDescription = null, modifier = Modifier.size(20.dp))
                Spacer(Modifier.width(12.dp))
                Text(text.supportDevelopment, fontWeight = FontWeight.ExtraBold, fontSize = 16.sp)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun DeveloperSocials(
    contributor: AboutContributor,
    text: AboutScreenText,
    onOpenUrl: (String) -> Unit,
) {
    ButtonGroup(
        overflowIndicator = { menuState -> ButtonGroupDefaults.OverflowIndicator(menuState) },
        expandedRatio = 1f,
        horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth().height(48.dp),
    ) {
        customItem(
            buttonGroupContent = {
                FilledTonalButton(
                    onClick = { contributor.websiteUrl?.let(onOpenUrl) },
                    enabled = contributor.websiteUrl != null,
                    shapes = ButtonDefaults.shapes(
                        shape = ButtonGroupDefaults.connectedLeadingButtonShape,
                        pressedShape = ButtonGroupDefaults.connectedLeadingButtonPressShape,
                    ),
                    modifier = Modifier.weight(1f).height(48.dp),
                ) {
                    Icon(Icons.Outlined.Language, contentDescription = text.openWebsite)
                }
            },
            menuContent = { menuState ->
                DropdownMenuItem(
                    leadingIcon = { Icon(Icons.Outlined.Language, contentDescription = null) },
                    text = { Text(text.openWebsite) },
                    enabled = contributor.websiteUrl != null,
                    onClick = {
                        contributor.websiteUrl?.let(onOpenUrl)
                        menuState.dismiss()
                    },
                )
            },
        )
        customItem(
            buttonGroupContent = {
                FilledTonalButton(
                    onClick = { contributor.githubUrl?.let(onOpenUrl) },
                    enabled = contributor.githubUrl != null,
                    shapes = ButtonDefaults.shapes(
                        shape = ButtonGroupDefaults.connectedTrailingButtonShape,
                        pressedShape = ButtonGroupDefaults.connectedTrailingButtonPressShape,
                    ),
                    modifier = Modifier.weight(1f).height(48.dp),
                ) {
                    Icon(Icons.Brand.Github, contentDescription = text.openGithubProfile)
                }
            },
            menuContent = { menuState ->
                DropdownMenuItem(
                    leadingIcon = { Icon(Icons.Brand.Github, contentDescription = null) },
                    text = { Text(text.openGithubProfile) },
                    enabled = contributor.githubUrl != null,
                    onClick = {
                        contributor.githubUrl?.let(onOpenUrl)
                        menuState.dismiss()
                    },
                )
            },
        )
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun ContributorAvatar(
    contributor: AboutContributor,
    sizeDp: Int,
    modifier: Modifier = Modifier,
) {
    val shape: Shape = when (contributor.avatarShape) {
        AboutAvatarShape.COOKIE_4_SIDED -> MaterialShapes.Cookie4Sided.toShape()
        AboutAvatarShape.COOKIE_6_SIDED -> MaterialShapes.Cookie6Sided.toShape()
        AboutAvatarShape.COOKIE_9_SIDED -> MaterialShapes.Cookie9Sided.toShape()
        AboutAvatarShape.BURST -> MaterialShapes.Burst.toShape()
        AboutAvatarShape.PUFFY_DIAMOND -> MaterialShapes.PuffyDiamond.toShape()
        AboutAvatarShape.SUNNY -> MaterialShapes.Sunny.toShape()
        AboutAvatarShape.GEM -> MaterialShapes.Gem.toShape()
    }
    Surface(
        modifier = modifier.size(sizeDp.dp),
        shape = shape,
        color = MaterialTheme.colorScheme.surfaceContainerHighest,
        tonalElevation = 4.dp,
    ) {
        AsyncImage(
            model = contributor.avatarUrl,
            contentDescription = contributor.name,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxSize(),
            shape = shape,
        )
    }
}

@Composable
private fun ContributorsSection(
    contributors: List<AboutContributor>,
    text: AboutScreenText,
    onOpenUrl: (String) -> Unit,
) {
    AboutSection(
        title = text.contributors,
        itemCount = contributors.size,
        itemContent = { index ->
            val contributor = contributors[index]
            Row(
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                ContributorAvatar(
                    contributor = contributor,
                    sizeDp = 48,
                )
                Spacer(Modifier.width(16.dp))
                Column(Modifier.weight(1f)) {
                    Text(text = contributor.name, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                    Text(
                        text = text.roleLabel(contributor.role),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (contributor.sponsorUrl != null) {
                    IconButton(
                        onClick = { onOpenUrl(contributor.sponsorUrl) },
                        modifier = Modifier.size(48.dp),
                    ) {
                        Surface(
                            shape = CircleShape,
                            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f),
                            modifier = Modifier.size(36.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    Icons.Filled.Favorite,
                                    contentDescription = "Sponsor",
                                    modifier = Modifier.size(20.dp),
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }
            }
        },
        onClick = { index -> contributors[index].githubUrl?.let(onOpenUrl) },
    )
}

@Composable
private fun InfoBadge(
    label: String,
    containerColor: androidx.compose.ui.graphics.Color,
    contentColor: androidx.compose.ui.graphics.Color,
) {
    Surface(shape = RoundedCornerShape(8.dp), color = containerColor) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.Bold,
            color = contentColor,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
        )
    }
}

@Composable
private fun AboutSection(
    title: String,
    itemCount: Int,
    itemContent: @Composable (Int) -> Unit,
    onClick: (Int) -> Unit,
) {
    Column(Modifier.fillMaxWidth()) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Medium),
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(top = 8.dp, bottom = 12.dp),
        )
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            repeat(itemCount) { index ->
                val shape =
                    RoundedCornerShape(
                        topStart = if (index == 0) 20.dp else 5.dp,
                        topEnd = if (index == 0) 20.dp else 5.dp,
                        bottomStart = if (index == itemCount - 1) 20.dp else 5.dp,
                        bottomEnd = if (index == itemCount - 1) 20.dp else 5.dp,
                    )
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { onClick(index) },
                    shape = shape,
                    color = MaterialTheme.colorScheme.surfaceContainer,
                ) {
                    itemContent(index)
                }
            }
        }
    }
}
