package me.lampu.lampcord.shared.update

import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption

actual fun installUpdate(
    target: UpdateTarget,
    downloadedFile: String,
    version: String,
): InstallResult {
    val source = File(downloadedFile)
    if (!source.isFile) return InstallResult.Failed("The downloaded build is missing")

    return when (target) {
        UpdateTarget.LINUX -> installLinux(source, version)
        UpdateTarget.WINDOWS -> installWindows(source)
        UpdateTarget.MACOS -> InstallResult.Unsupported(
            "macOS builds ship as a disk image, so replace the app bundle and relaunch"
        )
        UpdateTarget.ANDROID -> InstallResult.Unsupported("Android installs through the package installer")
    }
}

private fun installWindows(installer: File): InstallResult = runCatching {
    ProcessBuilder(
        installer.absolutePath,
        "/S",
        "--pid",
        ProcessHandle.current().pid().toString()
    ).directory(installer.parentFile).start()
    InstallResult.Started
}.getOrElse { InstallResult.Failed(it.message ?: "The installer could not be started") }

private fun installLinux(source: File, version: String): InstallResult = runCatching {
    val current = locateRunningAppImage() ?: return InstallResult.Unsupported(
        "This build was not started from an AppImage, so it cannot replace itself. " +
            "Update through the package or store you installed it from."
    )
    val destination = current.toPath().toAbsolutePath()
    val staged = destination.resolveSibling("${destination.fileName}.$version.new")

    Files.copy(source.toPath(), staged, StandardCopyOption.REPLACE_EXISTING)
    staged.toFile().setExecutable(true, false)

    val script = File.createTempFile("lampcord-update", ".sh")
    script.writeText(
        """
        #!/bin/sh
        while kill -0 ${ProcessHandle.current().pid()} 2>/dev/null; do sleep 0.3; done
        mv -f "${staged.fileName}" "${destination.fileName}"
        chmod +x "${destination.fileName}"
        rm -f "$0"
        """.trimIndent() + "\n"
    )
    script.setExecutable(true, false)
    ProcessBuilder("/bin/sh", script.absolutePath).start()

    InstallResult.Started
}.getOrElse { InstallResult.Failed(it.message ?: "The update could not be staged") }

private fun locateRunningAppImage(): File? {
    val env = System.getenv("APPIMAGE")
    return env?.takeIf { it.isNotBlank() }?.let(::File)?.takeIf { it.isFile }
}
