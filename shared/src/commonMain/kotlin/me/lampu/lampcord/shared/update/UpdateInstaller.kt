package me.lampu.lampcord.shared.update

sealed interface InstallResult {
    data object Started : InstallResult
    data class Unsupported(val reason: String) : InstallResult
    data class Failed(val reason: String) : InstallResult
}

expect fun installUpdate(
    target: UpdateTarget,
    downloadedFile: String,
    version: String,
): InstallResult
