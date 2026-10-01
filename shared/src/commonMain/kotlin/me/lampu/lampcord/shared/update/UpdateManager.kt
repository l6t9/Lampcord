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
    data class Downloading(val version: String, val progress: Int) : UpdateState
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
    /** Null on platforms with nothing to update from, such as iOS. */
    private val target: UpdateTarget?,
    private val now: () -> Long = { getCurrentTimeMillis() },
) {
    private val mutex = Mutex()
    private val mutableState = MutableStateFlow<UpdateState>(UpdateState.Idle)

    private val downloadClient: HttpClient by lazy {
        HttpClient(httpClientEngine()) {
            expectSuccess = true
            install(HttpTimeout) {
                connectTimeoutMillis = 30_000
                socketTimeoutMillis = 120_000
                requestTimeoutMillis = 30 * 60_000
            }
        }
    }

    val state: StateFlow<UpdateState> = mutableState.asStateFlow()

    suspend fun check(channel: UpdateChannel, force: Boolean = false): UpdateState = mutex.withLock {
        val target = target ?: return@withLock UpdateState.UpToDate(now()).also { mutableState.value = it }
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

        val candidates = release.assets.filter { target.matchesName(it) }
        val asset = target.installerExtension
            ?.let { suffix -> candidates.firstOrNull { it.name.endsWith(suffix, ignoreCase = true) } }
            ?: candidates.firstOrNull()
        if (asset == null) {
            return@withLock fail("No ${target.extension} build in ${release.tagName}", release)
        }

        val offered = release.assetVersion(asset, target) ?: release.version
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
            mutableState.value = UpdateState.Downloading(release.version, 0)
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

    private suspend fun fetchRelease(channel: UpdateChannel): GitHubRelease? {
        val url = when (channel) {
            UpdateChannel.STABLE -> "$API_BASE/releases/latest"
            UpdateChannel.NIGHTLY -> "$API_BASE/releases/tags/${channel.tag}"
        }
        val body = client.get(url) {
            header("Accept", "application/vnd.github+json")
            header("X-GitHub-Api-Version", "2022-11-28")
            header("User-Agent", "Lampcord/${currentVersion()}")
        }.bodyAsText()
        return json.decodeFromString<GitHubRelease>(body).takeIf { !it.draft }
    }

    private fun fail(reason: String, release: GitHubRelease?): UpdateState =
        UpdateState.Failed(reason, release).also { mutableState.value = it }
}
