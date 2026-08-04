package com.example.lampcord.shared.utils

import java.awt.Toolkit
import java.awt.datatransfer.DataFlavor
import java.io.File
import java.nio.file.Files
import java.awt.Image
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import java.awt.datatransfer.StringSelection

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

actual fun setClipboardText(text: String) {
    val selection = StringSelection(text)
    Toolkit.getDefaultToolkit().systemClipboard.setContents(selection, selection)
}
