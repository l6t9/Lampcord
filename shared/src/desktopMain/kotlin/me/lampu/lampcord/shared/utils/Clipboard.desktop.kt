package me.lampu.lampcord.shared.utils

import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.text.AnnotatedString
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import java.awt.Toolkit
import java.awt.datatransfer.DataFlavor
import java.io.File
import java.nio.file.Files
import java.awt.Image
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO

private var clipboard: androidx.compose.ui.platform.Clipboard? = null
private var clipboardScope: CoroutineScope? = null

@Composable
actual fun ProvideClipboard() {
    val current = LocalClipboard.current
    val scope = rememberCoroutineScope()
    DisposableEffect(current) {
        clipboard = current
        clipboardScope = scope
        onDispose {
            clipboard = null
            clipboardScope = null
        }
    }
}

actual fun getClipboardFiles(): List<Pair<String, ByteArray>> {
    val clipboard = Toolkit.getDefaultToolkit().systemClipboard
    return try {
        if (clipboard.isDataFlavorAvailable(DataFlavor.javaFileListFlavor)) {
            @Suppress("UNCHECKED_CAST")
            val files = clipboard.getData(DataFlavor.javaFileListFlavor) as List<File>
            files.map { it.name to Files.readAllBytes(it.toPath()) }
        } else if (clipboard.isDataFlavorAvailable(DataFlavor.imageFlavor)) {
            val image = clipboard.getData(DataFlavor.imageFlavor) as Image
            val bufferedImage = if (image is BufferedImage) {
                image
            } else {
                val bImg = BufferedImage(
                    image.getWidth(null),
                    image.getHeight(null),
                    BufferedImage.TYPE_INT_ARGB
                )
                val g = bImg.createGraphics()
                g.drawImage(image, 0, 0, null)
                g.dispose()
                bImg
            }
            val baos = ByteArrayOutputStream()
            ImageIO.write(bufferedImage, "png", baos)
            listOf("pasted_image.png" to baos.toByteArray())
        } else {
            emptyList()
        }
    } catch (e: Exception) {
        emptyList()
    }
}

@OptIn(ExperimentalComposeUiApi::class)
actual fun setClipboardText(text: String) {
    val target = clipboard
    val scope = clipboardScope
    if (target == null || scope == null) return
    scope.launch { target.setClipEntry(ClipEntry(AnnotatedString(text))) }
}
