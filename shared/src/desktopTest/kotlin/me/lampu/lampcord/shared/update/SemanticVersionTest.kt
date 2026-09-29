package me.lampu.lampcord.shared.update

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SemanticVersionTest {

    private fun version(value: String): SemanticVersion =
        requireNotNull(SemanticVersion.parse(value)) { "expected $value to parse" }


    @Test
    fun `nightly counters order numerically, not as strings`() {
        val a9 = version("1.0.0-a9")
        val a10 = version("1.0.0-a10")
        val a100 = version("1.0.0-a100")

        assertTrue(a9 < a10, "a9 must sort before a10")
        assertTrue(a10 < a100, "a10 must sort before a100")
    }

    @Test
    fun `a stable release outranks any prerelease of the same version`() {
        val stable = version("1.0.0")
        val nightly = version("1.0.0-a9")

        assertTrue(nightly < stable)
        assertTrue(stable > nightly)
    }

    @Test
    fun `a newer version outranks a prerelease of an older one`() {
        val olderPrerelease = version("0.9.0-a99")
        val newerStable = version("1.0.0")

        assertTrue(olderPrerelease < newerStable)
    }

    @Test
    fun `shorter prerelease lists sort before their extensions`() {
        val alpha = version("1.0.0-alpha")
        val alphaOne = version("1.0.0-alpha.1")

        assertTrue(alpha < alphaOne)
    }

    @Test
    fun `numeric identifiers rank below alphanumeric ones`() {
        val numeric = version("1.0.0-1")
        val alphanumeric = version("1.0.0-alpha")

        assertTrue(numeric < alphanumeric)
    }

    @Test
    fun `build metadata is ignored when ordering`() {
        val plain = version("1.0.0+1")
        val stamped = version("1.0.0+2")

        assertEquals(0, plain.compareTo(stamped))
    }

    @Test
    fun `tags, whitespace and plus metadata all parse`() {
        assertEquals("1.0.0", version("v1.0.0").toString())
        assertEquals("1.2.3", version("  1.2.3  ").toString())
        assertEquals("1.0.0+build.5", version("1.0.0+build.5").toString())
        assertEquals("1.0.0-a.1", version("1.0.0-a.1").toString())
    }

    @Test
    fun `malformed versions are rejected rather than guessed`() {
        assertNull(SemanticVersion.parse(""))
        assertNull(SemanticVersion.parse("1"))
        assertNull(SemanticVersion.parse("1.0"))
        assertNull(SemanticVersion.parse("1.0.0.0"))
        assertNull(SemanticVersion.parse("v1.0"))
        assertNull(SemanticVersion.parse("01.0.0"), "leading zeroes are not valid semver")
        assertNull(SemanticVersion.parse("1.0.0-"))
    }

    @Test
    fun `equal versions compare equal regardless of how they were written`() {
        assertEquals(version("v1.2.3"), version("1.2.3"))
        assertEquals(0, version("1.0.0-a9").compareTo(version("1.0.0-a9")))
    }

    @Test
    fun `isNewer refuses to compare against a version it cannot parse`() {
        assertTrue(SemanticVersion.isNewer("1.0.1", "1.0.0"))
        assertFalse(SemanticVersion.isNewer("1.0.0", "1.0.0"))
        assertFalse(SemanticVersion.isNewer("1.0.0", "1.0.1"))
        assertFalse(SemanticVersion.isNewer("garbage", "1.0.0"))
        assertFalse(SemanticVersion.isNewer("1.0.0", "garbage"), "a bad current version must not offer updates")
    }
}
