package me.lampu.lampcord.shared.update

import okio.ByteString.Companion.toByteString
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class UpdateAssetTest {

    private fun asset(name: String, state: String = "uploaded") = GitHubAsset(
        name = name,
        state = state,
        size = 1234,
        digest = "sha256:" + "a".repeat(64),
        downloadUrl = "https://example.invalid/$name",
    )

    @Test
    fun `assets are matched per target and extension`() {
        val linux = asset("lampcord-desktop-linux-x64-1.0.0.AppImage")
        val windows = asset("lampcord-desktop-windows-x64-1.0.0.zip")
        val android = asset("lampcord-android-1.0.0.apk")

        assertTrue(UpdateTarget.LINUX.matches(linux))
        assertFalse(UpdateTarget.LINUX.matches(windows))
        assertFalse(UpdateTarget.LINUX.matches(android))

        assertTrue(UpdateTarget.WINDOWS.matches(windows))
        assertFalse(UpdateTarget.WINDOWS.matches(linux))

        assertTrue(UpdateTarget.ANDROID.matches(android))
        assertFalse(UpdateTarget.ANDROID.matches(linux))
    }

    @Test
    fun `debug and unsigned builds are never offered`() {
        listOf(
            "lampcord-desktop-linux-x64-1.0.0-debug.AppImage",
            "lampcord-desktop-linux-x64-1.0.0-unsigned.AppImage",
            "lampcord-android-1.0.0-debug.apk",
            "lampcord-android-1.0.0-unaligned.apk",
            "lampcord-android-1.0.0-benchmark.apk",
        ).forEach { name ->
            val target = if (name.contains("android")) UpdateTarget.ANDROID else UpdateTarget.LINUX
            assertFalse(target.matches(asset(name)), "$name must not be offered")
        }
    }

    @Test
    fun `a source archive is not mistaken for a build`() {
        assertFalse(UpdateTarget.LINUX.matches(asset("lampcord-desktop-linux-x64-1.0.0-source.AppImage")))
    }

    @Test
    fun `release metadata deserialises from the GitHub shape`() {
        val json = """
            {
              "tag_name": "v1.2.3",
              "draft": false,
              "prerelease": true,
              "html_url": "https://github.com/l6t9/Lampcord/releases/tag/v1.2.3",
              "body": "notes",
              "assets": [
                {
                  "name": "lampcord-desktop-linux-x64-1.2.3.AppImage",
                  "state": "uploaded",
                  "size": 42,
                  "digest": "sha256:${"b".repeat(64)}",
                  "browser_download_url": "https://example.invalid/a.AppImage"
                }
              ]
            }
        """.trimIndent()

        val release = UpdateModels.json.decodeFromString<GitHubRelease>(json)
        assertEquals("v1.2.3", release.tagName)
        assertEquals("1.2.3", release.version)
        assertTrue(release.prerelease)
        assertEquals(1, release.assets.size)
        assertTrue(UpdateTarget.LINUX.matches(release.assets.first()))
    }

    @Test
    fun `digests are verified and a malformed one is rejected`() {
        val bytes = "lampcord".encodeToByteArray()
        val good = "sha256:" + bytes.toByteString().sha256().hex()

        assertTrue(downloadMatchesDigest(bytes, good))
        assertTrue(downloadMatchesDigest(bytes, good.uppercase()))
        assertFalse(downloadMatchesDigest(bytes, "sha256:" + "0".repeat(64)))
        assertFalse(downloadMatchesDigest(bytes, null))
        assertFalse(downloadMatchesDigest(bytes, "not-a-digest"))
        assertFalse(downloadMatchesDigest(bytes, "md5:" + "a".repeat(32)))
    }

    @Test
    fun `the rolling nightly tag takes its version from the asset name`() {
        val nightly = GitHubRelease(tagName = "nightly", prerelease = true)
        val asset = asset("lampcord-desktop-linux-x64-1.0.0-a9.AppImage")

        assertEquals("nightly", nightly.version)
        assertNull(SemanticVersion.parse(nightly.version))
        assertEquals("1.0.0-a9", nightly.assetVersion(asset, UpdateTarget.LINUX))
        assertTrue(SemanticVersion.isNewer("1.0.0-a9", "1.0.0-a8"))
        assertFalse(SemanticVersion.isNewer("1.0.0-a9", "1.0.0-a9"))
    }

    @Test
    fun `a version is recovered from every real asset name`() {
        val release = GitHubRelease(tagName = "v1.0.0")
        listOf(
            "lampcord-android-1.0.0.apk" to "1.0.0",
            "lampcord-desktop-linux-x64-1.0.0-a9.AppImage" to "1.0.0-a9",
            "lampcord-desktop-windows-x64-1.2.3-beta.2.zip" to "1.2.3-beta.2",
        ).forEach { (name, expected) ->
            val target = when {
                name.contains("android") -> UpdateTarget.ANDROID
                name.contains("windows") -> UpdateTarget.WINDOWS
                else -> UpdateTarget.LINUX
            }
            assertEquals(expected, release.assetVersion(asset(name), target), name)
        }
    }

    @Test
    fun `unuploaded assets are not selectable`() {
        assertFalse(
            UpdateTarget.LINUX.matches(
                asset("lampcord-desktop-linux-x64-1.0.0.AppImage", state = "new")
            )
        )
    }
}
