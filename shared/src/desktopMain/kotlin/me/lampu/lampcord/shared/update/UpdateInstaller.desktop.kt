package me.lampu.lampcord.shared.update

import java.io.File
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import kotlin.system.exitProcess

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
    exitProcess(0)
    InstallResult.Started
}.getOrElse { InstallResult.Failed(it.message ?: "The installer could not be started") }

private fun installLinux(source: File, version: String): InstallResult = runCatching {
    val current = runningAppImagePath()?.let(::File) ?: return InstallResult.Unsupported(
        "This build was not started from an AppImage, so it cannot replace itself. " +
            "Update through the package or store you installed it from."
    )
    val destination = current.toPath().toAbsolutePath()
    val staged = destination.resolveSibling("${destination.fileName}.$version.new")
    val log = File(current.parentFile, ".${current.name}.update.log")

    Files.copy(source.toPath(), staged, StandardCopyOption.REPLACE_EXISTING)
    staged.toFile().setExecutable(true, false)

    val script = File.createTempFile("lampcord-update", ".sh")
    script.writeText(
        listOf(
            "#!/bin/sh",
            "log=${shellQuote(log.absolutePath)}",
            "staged=${shellQuote(staged.toString())}",
            "dest=${shellQuote(destination.toString())}",
            "while kill -0 ${ProcessHandle.current().pid()} 2>/dev/null; do sleep 0.3; done",
            "",
            "attempt=0",
            "while [ \"\$attempt\" -lt 40 ]; do",
            "  if mv -f \"\$staged\" \"\$dest\" 2>>\"\$log\"; then break; fi",
            "  attempt=\$((attempt + 1))",
            "  sleep 0.5",
            "done",
            "",
            "if [ \"\$attempt\" -ge 40 ]; then",
            "  echo \"could not replace the AppImage\" >> \"\$log\"",
            "  rm -f \"\$staged\"",
            "  exit 1",
            "fi",
            "",
            "chmod +x \"\$dest\"",
            "echo \"replaced with $version at \$(date)\" >> \"\$log\"",
            "rm -f \"\$0\"",
            "nohup \"\$dest\" >/dev/null 2>&1 &",
            ""
        ).joinToString("\n")
    )

    ProcessBuilder("/bin/sh", script.absolutePath)
        .redirectOutput(ProcessBuilder.Redirect.appendTo(log))
        .redirectError(ProcessBuilder.Redirect.appendTo(log))
        .start()

    exitProcess(0)
    InstallResult.Started
}.getOrElse { InstallResult.Failed(it.message ?: "The update could not be staged") }

internal fun shellQuote(value: String): String = "'" + value.replace("'", "'\\''") + "'"
