package me.lampu.lampcord.shared.update

actual fun installUpdate(
    target: UpdateTarget,
    downloadedFile: String,
    version: String,
): InstallResult = InstallResult.Unsupported("iOS installs through the App Store")
