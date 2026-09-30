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
import java.nio.file.Files
import javax.imageio.ImageIO

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
    val clipboard = systemClipboard() ?: return emptyList()
    val transferable = try {
        clipboard.getContents(null)
    } catch (e: Exception) {
        Logging.e(TAG, "Could not read the clipboard", e)
        null
    } ?: return emptyList()

    if (transferable.isDataFlavorSupported(DataFlavor.javaFileListFlavor)) {
        return readPastedFiles(transferable)
    }
    return readPastedImage(transferable)?.let(::listOf) ?: emptyList()
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
    for (flavor in IMAGE_FLAVORS) {
        if (!transferable.isDataFlavorSupported(flavor)) continue
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