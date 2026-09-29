package me.lampu.lampcord.shared.update

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class GitHubAsset(
    val name: String,
    val state: String = "",
    val size: Long = 0,
    val digest: String? = null,
    @SerialName("browser_download_url") val downloadUrl: String,
)

@Serializable
data class GitHubRelease(
    @SerialName("tag_name") val tagName: String = "",
    val name: String? = null,
    val body: String? = null,
    val draft: Boolean = false,
    val prerelease: Boolean = false,
    @SerialName("html_url") val htmlUrl: String = "",
    @SerialName("published_at") val publishedAt: String? = null,
    val assets: List<GitHubAsset> = emptyList(),
) {
    val version: String get() = tagName.removePrefix("v")
}

/**
 * The nightly release reuses a single rolling `nightly` tag, so the tag itself carries no version
 * and the real one has to come out of the asset name.
 */
fun GitHubRelease.assetVersion(asset: GitHubAsset, target: UpdateTarget): String? {
    val name = asset.name
    val start = name.lastIndexOf("lampcord-")
    if (start < 0) return null
    var tail = name.substring(start + "lampcord-".length)
    if (tail.endsWith(target.extension, ignoreCase = true)) {
        tail = tail.dropLast(target.extension.length)
    }
    return VERSION_IN_NAME.find(tail)?.groupValues?.get(1)
}

private val VERSION_IN_NAME = Regex("""(\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)""")

enum class UpdateTarget(val assetInfix: String?, val extension: String, val installerExtension: String? = null) {
    ANDROID("android", "apk"),
    LINUX("desktop-linux-x64", "AppImage"),
    WINDOWS("desktop-windows-x64", "zip", "-setup.exe"),
    MACOS("desktop-macos", "dmg");

    fun matches(asset: GitHubAsset): Boolean {
        if (!matchesExtension(asset)) return false
        return matchesName(asset)
    }

    fun matchesName(asset: GitHubAsset): Boolean {
        val infix = assetInfix ?: return false
        if (asset.state != "uploaded") return false
        if (BLOCKED_MARKERS.any { asset.name.contains(it, ignoreCase = true) }) return false
        return asset.name.contains(infix, ignoreCase = true)
    }

    fun matchesExtension(asset: GitHubAsset): Boolean {
        val infix = assetInfix ?: return false
        if (BLOCKED_MARKERS.any { asset.name.contains(it, ignoreCase = true) }) return false
        return asset.name.endsWith(extension, ignoreCase = true) ||
            asset.name.contains(infix, ignoreCase = true)
    }

    private companion object {
        val BLOCKED_MARKERS = listOf("debug", "unsigned", "unaligned", "test", "benchmark", "source", "symbols")
    }
}

enum class UpdateChannel(val id: String, val tag: String) {
    STABLE("stable", "latest"),
    NIGHTLY("nightly", "nightly");

    val isPrerelease: Boolean get() = this == NIGHTLY
}
