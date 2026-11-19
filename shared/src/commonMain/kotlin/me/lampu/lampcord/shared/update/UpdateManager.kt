package me.lampu.lampcord.shared.update

import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.statement.bodyAsChannel
import io.ktor.client.statement.bodyAsText
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json
import me.lampu.lampcord.shared.api.httpClientEngine
import me.lampu.lampcord.shared.utils.getCurrentTimeMillis

private const val API_BASE = "https://api.github.com/repos/l6t9/Lampcord"
private const val MAX_ASSET_BYTES = 512L * 1024 * 1024
private const val CHECK_INTERVAL_MILLIS = 60L * 60 * 1000

sealed interface UpdateState {
    data object Idle : UpdateState
    data object Checking : UpdateState
    data class UpToDate(val checkedAt: Long) : UpdateState
    data class Available(
        val release: GitHubRelease,
        val asset: GitHubAsset,
        val version: String,
        val changelog: String,
    ) : UpdateState
    data class Downloading(val version: String) : UpdateState
    data class ReadyToInstall(
        val release: GitHubRelease,
        val version: String,
        val file: String,
    ) : UpdateState
    data class Failed(val reason: String, val release: GitHubRelease? = null) : UpdateState
}

class UpdateManager(
    private val client: HttpClient,
    private val json: Json,
    private val currentVersion: () -> String,

    private val target: UpdateTarget?,
    private val now: () -> Long = { getCurrentTimeMillis() },
) {
    private val mutex = Mutex()
    private val mutableState = MutableStateFlow<UpdateState>(UpdateState.Idle)

    private val downloadClientDelegate = lazy {
        HttpClient(httpClientEngine()) {
            expectSuccess = true
            install(HttpTimeout) {
                connectTimeoutMillis = 30_000
                socketTimeoutMillis = 120_000
                requestTimeoutMillis = 30 * 60_000
            }
        }
    }

    private val downloadClient: HttpClient by downloadClientDelegate

    val state: StateFlow<UpdateState> = mutableState.asStateFlow()

    suspend fun check(channel: UpdateChannel, force: Boolean = false): UpdateState = mutex.withLock {
        val target = target ?: return@withLock UpdateState.UpToDate(now()).also { mutableState.value = it }
        // A download that is running or already finished is visible to the user. Re-entering
        // the settings screen must not replace it just because it is not an UpToDate state.
        if (mutableState.value is UpdateState.Downloading ||
            mutableState.value is UpdateState.ReadyToInstall
        ) return@withLock mutableState.value
        val lastCheck = (mutableState.value as? UpdateState.UpToDate)?.checkedAt
        if (!force && lastCheck != null && now() - lastCheck < CHECK_INTERVAL_MILLIS) {
            return@withLock mutableState.value
        }

        mutableState.value = UpdateState.Checking
        val release = runCatching { fetchRelease(channel) }.getOrNull()
        if (release == null) {
            return@withLock fail("Could not reach the release feed", null)
        }
        val matchesChannel = if (channel.isPrerelease) release.prerelease else !release.prerelease
        if (!matchesChannel) {
            return@withLock UpdateState.UpToDate(now()).also { mutableState.value = it }
        }

        val asset = release.bestAsset(target)
        if (asset == null) {
            return@withLock fail("No ${target.extension} build in ${release.tagName}", release)
        }

        val offered = release.offeredVersion(asset, target)
        if (!SemanticVersion.isNewer(offered, currentVersion())) {
            return@withLock UpdateState.UpToDate(now()).also { mutableState.value = it }
        }

        UpdateState.Available(release, asset, offered, release.releaseNotes()).also {
            mutableState.value = it
        }
    }

    suspend fun download(): UpdateState = mutex.withLock {
        val target = target ?: return@withLock mutableState.value
        val available = mutableState.value as? UpdateState.Available
            ?: return@withLock mutableState.value
        val release = available.release
        val asset = available.asset

        if (asset.size !in 1..MAX_ASSET_BYTES) {
            return@withLock fail("That build reports an implausible size", release)
        }
        if (!DIGEST_PATTERN.matches(asset.digest.orEmpty())) {
            return@withLock fail("That release is missing a checksum", release)
        }

        return@withLock try {
            mutableState.value = UpdateState.Downloading(release.version)
            val channel = downloadClient.get(asset.downloadUrl) {
                header("User-Agent", "Lampcord/${currentVersion()}")
            }.bodyAsChannel()

            when (
                val outcome = storeDownload(
                    version = release.version,
                    target = target,
                    expectedSize = asset.size,
                    advertisedDigest = asset.digest,
                    source = channel,
                )
            ) {
                is StoreOutcome.Stored ->
                    UpdateState.ReadyToInstall(release, available.version, outcome.path).also {
                        mutableState.value = it
                    }

                StoreOutcome.SizeMismatch -> fail("The download did not match the advertised size", release)
                StoreOutcome.DigestMismatch -> fail("The download failed its checksum", release)
                is StoreOutcome.Failed -> fail(outcome.reason, release)
            }
        } catch (e: Exception) {
            fail(e.message ?: "The download failed", release)
        }
    }

    fun reset() {
        mutableState.value = UpdateState.Idle
    }

    fun close() {
        if (downloadClientDelegate.isInitialized()) {
            downloadClient.close()
        }
    }

    private suspend fun fetchRelease(channel: UpdateChannel): GitHubRelease? =
        when (channel) {
            UpdateChannel.STABLE -> fetchLatestStable()
            UpdateChannel.NIGHTLY -> fetchLatestNightly()
        }

    private suspend fun fetchLatestStable(): GitHubRelease? {
        val release = json.decodeFromString<GitHubRelease>(get("$API_BASE/releases/latest"))
        return release.takeIf { !it.draft }
    }

    private suspend fun fetchLatestNightly(): GitHubRelease? {
        val target = target ?: return null
        val releases = json.decodeFromString<List<GitHubRelease>>(get("$API_BASE/releases?per_page=100"))
        return releases.latestNightlyFor(target)
    }

    private suspend fun get(url: String): String =
        client.get(url) {
            header("Accept", "application/vnd.github+json")
            header("X-GitHub-Api-Version", "2022-11-28")
            header("User-Agent", "Lampcord/${currentVersion()}")
        }.bodyAsText()

    private fun fail(reason: String, release: GitHubRelease?): UpdateState =
        UpdateState.Failed(reason, release).also { mutableState.value = it }
}
