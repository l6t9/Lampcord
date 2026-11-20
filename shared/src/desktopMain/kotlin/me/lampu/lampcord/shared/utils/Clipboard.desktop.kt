package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import java.awt.Image
import java.awt.Toolkit
import java.awt.datatransfer.DataFlavor
import java.awt.datatransfer.StringSelection
import java.awt.datatransfer.Transferable
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.InputStream
import java.net.URI
import java.nio.file.Files
import java.util.Base64
import javax.imageio.ImageIO
import java.nio.file.Path
import kotlin.io.path.name

private const val TAG = "Clipboard"
private const val IMAGE_NAME = "pasted_image.png"

private val IMAGE_FLAVORS = listOf(
    DataFlavor.imageFlavor,
    DataFlavor("image/png"),
    DataFlavor("image/jpeg"),
)

@Composable
actual fun ProvideClipboard() = Unit

actual fun setClipboardText(text: String) {
    val clipboard = systemClipboard() ?: return
    try {
        clipboard.setContents(StringSelection(text), null)
    } catch (e: Exception) {
        Logging.e(TAG, "Could not write to the clipboard", e)
    }
}

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> {
    val isWindows = System.getProperty("os.name").lowercase().contains("win")
    val clipboard = systemClipboard()
    val transferable = if (clipboard != null) {
        try {
            clipboard.getContents(null)
        } catch (e: Exception) {
            Logging.e(TAG, "Could not read the clipboard", e)
            null
        }
    } else {
        null
    }

    if (transferable != null) {
        if (transferable.isDataFlavorSupported(DataFlavor.javaFileListFlavor)) {
            val files = readPastedFiles(transferable)
            if (files.isNotEmpty() || !isWindows) return files
        } else {
            val image = readPastedImage(transferable)
            if (image != null) return listOf(image)
        }
    }
    return if (isWindows) {
        readClipboardFromPowerShell() ?: emptyList()
    } else {
        readClipboardFilePathsFromCommand() ?: readClipboardImageFromCommand() ?: emptyList()
    }
}

private fun readPastedFiles(transferable: Transferable): List<Pair<String, ByteArray>> {
    val files = try {
        @Suppress("UNCHECKED_CAST")
        transferable.getTransferData(DataFlavor.javaFileListFlavor) as List<File>
    } catch (e: Exception) {
        Logging.e(TAG, "Could not read the pasted file list", e)
        return emptyList()
    }
    return files.mapNotNull { file ->
        try {
            file.name to Files.readAllBytes(file.toPath())
        } catch (e: Exception) {
            Logging.e(TAG, "Could not read the pasted file ${file.name}", e)
            null
        }
    }
}

private fun readPastedImage(transferable: Transferable): Pair<String, ByteArray>? {
    val flavors = transferable.transferDataFlavors.toList() + IMAGE_FLAVORS
    for (flavor in flavors.distinct()) {
        if (!transferable.isDataFlavorSupported(flavor)) continue
        if (!flavor.mimeType.startsWith("image/")) continue
        try {
            when (val data = transferable.getTransferData(flavor)) {
                is Image -> return IMAGE_NAME to encodePng(data.toBufferedImage())
                is InputStream -> return IMAGE_NAME to data.readBytes()
                is ByteArray -> return IMAGE_NAME to data
            }
        } catch (e: Exception) {
            Logging.e(TAG, "Could not read a pasted image as ${flavor.mimeType}", e)
        }
    }
    return null
}

private fun readClipboardFilePathsFromCommand(): List<Pair<String, ByteArray>>? {
    val commands = listOf(
        listOf("wl-paste", "--type", "text/uri-list"),
        listOf("xclip", "-selection", "clipboard", "-t", "text/uri-list", "-o"), // not tested on x11
    )
    val pathUriStringList = commands.firstNotNullOfOrNull { command ->
        try {
            val process = ProcessBuilder(command).start()
            val text = process.inputStream.bufferedReader().use { it.readText() }
            val exit = process.waitFor()
            if (exit == 0 && text.isNotEmpty()) text.trim().split("\n")
            else null
        } catch (e: Exception) {
            Logging.d(TAG, "Clipboard command unavailable: ${command.joinToString(" ")}")
            null
        }
    } ?: emptyList()
    if(pathUriStringList.isEmpty()) return null

    return pathUriStringList.mapNotNull { pathUriString ->
        try {
            val path = Path.of(URI(pathUriString.trim().trimEnd('\u0000')))
            val file = path.toFile()
            path.name to file.readBytes()
        } catch (e: Exception) {
            Logging.e(TAG, "Received an invalid file path from clipboard: $pathUriString", e)
            null
        }
    }
}

private fun readClipboardImageFromCommand(): List<Pair<String, ByteArray>>? {
    val commands = listOf(
        listOf("wl-paste", "--type", "image/png"),
        listOf("wl-paste", "--type", "image/jpeg"),
        listOf("xclip", "-selection", "clipboard", "-t", "image/png", "-o"),
        listOf("xclip", "-selection", "clipboard", "-t", "image/jpeg", "-o"),
    )
    for (command in commands) {
        try {
            val process = ProcessBuilder(command).start()
            val bytes = process.inputStream.readBytes()
            val exit = process.waitFor()
            if (exit == 0 && bytes.isNotEmpty()) {
                val name = if (command.contains("jpeg")) "pasted_image.jpg" else IMAGE_NAME
                return listOf(name to bytes)
            }
        } catch (e: Exception) {
            Logging.d(TAG, "Clipboard command unavailable: ${command.joinToString(" ")}")
        }
    }
    return null
}

private fun readClipboardFromPowerShell(): List<Pair<String, ByteArray>>? {
    val script = """
        Add-Type -AssemblyName System.Windows.Forms
        Add-Type -AssemblyName System.Drawing
        ${'$'}clip = [System.Windows.Forms.Clipboard]
        if (${'$'}clip.ContainsFileDropList()) {
            foreach (${'$'}path in ${'$'}clip.GetFileDropList()) {
                ${'$'}bytes = [System.Text.Encoding]::UTF8.GetBytes(${'$'}path)
                Write-Output ('FILE:' + [System.Convert]::ToBase64String(${'$'}bytes))
            }
        } elseif (${'$'}clip.ContainsImage()) {
            ${'$'}image = ${'$'}clip.GetImage()
            ${'$'}path = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), 'lampcord-paste-' + [guid]::NewGuid().ToString('N') + '.png')
            ${'$'}image.Save(${'$'}path, [System.Drawing.Imaging.ImageFormat]::Png)
            ${'$'}image.Dispose()
            ${'$'}bytes = [System.Text.Encoding]::UTF8.GetBytes(${'$'}path)
            Write-Output ('IMAGE:' + [System.Convert]::ToBase64String(${'$'}bytes))
        }
    """.trimIndent()
    return try {
        val process = ProcessBuilder("powershell.exe", "-NoProfile", "-NonInteractive", "-STA", "-Command", script)
            // Merge stderr so a chatty failure cannot fill the pipe while stdout is being read.
            .redirectErrorStream(true)
            .start()
        val output = process.inputStream.bufferedReader().use { it.readLines() }
        if (process.waitFor() != 0) {
            Logging.e(TAG, "PowerShell clipboard fallback failed: ${output.joinToString("\n").trim()}")
            return null
        }
        val files = mutableListOf<Pair<String, ByteArray>>()
        var imagePath: String? = null
        for (line in output) {
            when {
                line.startsWith("FILE:") -> {
                    val path = decodePowerShellPath(line.removePrefix("FILE:")) ?: continue
                    try {
                        val file = File(path)
                        files += file.name to Files.readAllBytes(file.toPath())
                    } catch (e: Exception) {
                        Logging.e(TAG, "Could not read the pasted file $path", e)
                    }
                }
                line.startsWith("IMAGE:") -> imagePath = decodePowerShellPath(line.removePrefix("IMAGE:"))
            }
        }
        imagePath?.let { path ->
            val file = File(path)
            try {
                files += IMAGE_NAME to Files.readAllBytes(file.toPath())
            } catch (e: Exception) {
                Logging.e(TAG, "Could not read the pasted image", e)
            } finally {
                file.delete()
            }
        }
        files.ifEmpty { null }
    } catch (e: Exception) {
        Logging.e(TAG, "Could not read the clipboard with PowerShell", e)
        null
    }
}

private fun decodePowerShellPath(value: String): String? = try {
    String(Base64.getDecoder().decode(value), Charsets.UTF_8)
} catch (e: IllegalArgumentException) {
    Logging.e(TAG, "Could not decode a clipboard path", e)
    null
}

private fun systemClipboard() = try {
    Toolkit.getDefaultToolkit().systemClipboard
} catch (e: Exception) {
    Logging.e(TAG, "The system clipboard is unavailable", e)
    null
}

private fun Image.toBufferedImage(): BufferedImage {
    if (this is BufferedImage) return this
    val copy = BufferedImage(getWidth(null), getHeight(null), BufferedImage.TYPE_INT_ARGB)
    val graphics = copy.createGraphics()
    try {
        graphics.drawImage(this, 0, 0, null)
    } finally {
        graphics.dispose()
    }
    return copy
}

private fun encodePng(image: BufferedImage): ByteArray = ByteArrayOutputStream().use { stream ->
    ImageIO.write(image, "png", stream)
    stream.toByteArray()
}