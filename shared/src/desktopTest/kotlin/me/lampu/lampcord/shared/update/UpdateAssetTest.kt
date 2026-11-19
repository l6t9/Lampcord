package me.lampu.lampcord.shared.update

import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.runBlocking
import okio.ByteString.Companion.toByteString
import java.io.File
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
    fun `digests are normalised and a malformed one is rejected`() {
        val bytes = "lampcord".encodeToByteArray()
        val hex = bytes.toByteString().sha256().hex()
        val good = "sha256:$hex"

        assertEquals(hex, normalizeDigest(good))
        assertEquals(hex, normalizeDigest(good.uppercase()))
        assertNull(normalizeDigest("sha256:" + "0".repeat(64) + "0"))
        assertNull(normalizeDigest(null))
        assertNull(normalizeDigest("not-a-digest"))
        assertNull(normalizeDigest("md5:" + "a".repeat(32)))
        assertNull(normalizeDigest("sha256:"))
    }

    @Test
    fun `a download is committed only when its size and digest both match`() = runBlocking {
        val payload = "lampcord release payload".encodeToByteArray()
        val digest = "sha256:" + payload.toByteString().sha256().hex()
        val version = "0.0.0-storetest"

        try {
            val stored = storeDownload(
                version = version,
                target = UpdateTarget.LINUX,
                expectedSize = payload.size.toLong(),
                advertisedDigest = digest,
                source = ByteReadChannel(payload),
            )
            assertTrue(stored is StoreOutcome.Stored, "was $stored")
            assertEquals(payload.size.toLong(), File(stored.path).length())

            val wrongSize = storeDownload(
                version = version,
                target = UpdateTarget.LINUX,
                expectedSize = payload.size.toLong() + 1,
                advertisedDigest = digest,
                source = ByteReadChannel(payload),
            )
            assertEquals(StoreOutcome.SizeMismatch, wrongSize)

            val wrongDigest = storeDownload(
                version = version,
                target = UpdateTarget.LINUX,
                expectedSize = payload.size.toLong(),
                advertisedDigest = "sha256:" + "0".repeat(64),
                source = ByteReadChannel(payload),
            )
            assertEquals(StoreOutcome.DigestMismatch, wrongDigest)

            val noDigest = storeDownload(
                version = version,
                target = UpdateTarget.LINUX,
                expectedSize = payload.size.toLong(),
                advertisedDigest = null,
                source = ByteReadChannel(payload),
            )
            assertEquals(StoreOutcome.DigestMismatch, noDigest)

            assertFalse(File(fileNameFor(version, UpdateTarget.LINUX)).exists())
        } finally {
            UpdateStorage.open(version, UpdateTarget.LINUX).abort()
        }
    }

    @Test
    fun `the nightly feed picks the newest build that has a downloadable asset`() {
        val older = GitHubRelease(
            tagName = "v1.0.4-nightly.150",
            prerelease = true,
            assets = listOf(asset("lampcord-android-1.0.4-nightly.150.apk")),
        )
        val skipped = GitHubRelease(
            tagName = "v1.0.4-nightly.154",
            prerelease = true,
            assets = emptyList(),
        )
        val newest = GitHubRelease(
            tagName = "v1.0.4-nightly.155",
            prerelease = true,
            assets = listOf(asset("lampcord-android-1.0.4-nightly.155.apk")),
        )
        val stable = GitHubRelease(
            tagName = "v1.0.5",
            prerelease = false,
            assets = listOf(asset("lampcord-android-1.0.5.apk")),
        )
        val draft = GitHubRelease(
            tagName = "v1.0.4-nightly.156",
            draft = true,
            prerelease = true,
            assets = listOf(asset("lampcord-android-1.0.4-nightly.156.apk")),
        )

        val feed = listOf(newest, skipped, stable, draft, older)
        assertEquals(newest, feed.latestNightlyFor(UpdateTarget.ANDROID))
    }

    @Test
    fun `the newest asset wins when a release carries more than one build`() {
        val release = GitHubRelease(
            tagName = "nightly",
            prerelease = true,
            assets = listOf(
                asset("lampcord-android-1.0.1.apk"),
                asset("lampcord-android-1.0.2-nightly.130.apk"),
            ),
        )

        assertEquals(
            "lampcord-android-1.0.2-nightly.130.apk",
            release.bestAsset(UpdateTarget.ANDROID)?.name,
        )
        assertEquals(
            "1.0.2-nightly.130",
            release.offeredVersion(release.bestAsset(UpdateTarget.ANDROID)!!, UpdateTarget.ANDROID),
        )
    }

    @Test
    fun `a versioned release tag beats a versionless asset name`() {
        val release = GitHubRelease(tagName = "v1.0.4-nightly.155", prerelease = true)
        val desktop = asset("lampcord-desktop-linux-x64-1.0.4.AppImage")

        assertEquals("1.0.4", release.assetVersion(desktop, UpdateTarget.LINUX))
        assertEquals("1.0.4-nightly.155", release.offeredVersion(desktop, UpdateTarget.LINUX))
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

    @Test
    fun `linux picks the AppImage from mixed assets regardless of order or other versions`() {
        val appImage = asset("lampcord-desktop-linux-x64-1.0.8.AppImage")
        val deb = asset("lampcord-desktop-linux-x64-1.0.9.deb")
        val jar = asset("lampcord-desktop-linux-x64-1.0.10.jar")
        val rpm = asset("lampcord-desktop-linux-x64-1.1.0.rpm")

        listOf(
            listOf(deb, appImage, jar, rpm),
            listOf(rpm, jar, deb, appImage),
            listOf(appImage, rpm, jar, deb),
        ).forEach { assets ->
            val release = GitHubRelease(tagName = "v1.0.8", assets = assets)
            assertEquals(appImage.name, release.bestAsset(UpdateTarget.LINUX)?.name, "assets=$assets")
        }
    }

    @Test
    fun `a nightly with only non-AppImage Linux assets is not offered to Linux`() {
        val debOnly = GitHubRelease(
            tagName = "v1.0.9-nightly.1",
            prerelease = true,
            assets = listOf(asset("lampcord-desktop-linux-x64-1.0.9.deb")),
        )
        val withAppImage = GitHubRelease(
            tagName = "v1.0.8-nightly.2",
            prerelease = true,
            assets = listOf(asset("lampcord-desktop-linux-x64-1.0.8.AppImage")),
        )

        assertEquals(withAppImage, listOf(debOnly, withAppImage).latestNightlyFor(UpdateTarget.LINUX))
    }

    @Test
    fun `windows still prefers the NSIS setup exe over the portable zip`() {
        val release = GitHubRelease(
            tagName = "v1.0.8",
            assets = listOf(
                asset("lampcord-desktop-windows-x64-1.0.8.zip"),
                asset("lampcord-desktop-windows-x64-1.0.8-setup.exe"),
            ),
        )

        assertEquals(
            "lampcord-desktop-windows-x64-1.0.8-setup.exe",
            release.bestAsset(UpdateTarget.WINDOWS)?.name,
        )
    }
}
