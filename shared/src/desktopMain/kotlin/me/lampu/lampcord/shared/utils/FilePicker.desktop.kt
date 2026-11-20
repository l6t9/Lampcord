package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.awt.FileDialog
import java.awt.Frame
import java.io.File
import java.nio.file.Files
import javax.swing.SwingUtilities

@Composable
actual fun FilePicker(
    show: Boolean,
    onFileSelected: (List<Pair<String, ByteArray>>) -> Unit,
    onDismiss: () -> Unit
) {
    if (show) {
        LaunchedEffect(Unit) {
            try {
                val os = System.getProperty("os.name").lowercase()
                val files = if (os.contains("linux")) {
                    withContext(Dispatchers.IO) { tryOpenZenity() }
                } else {
                    openAwtFileDialog()
                }
                if (!files.isNullOrEmpty()) {
                    onFileSelected(files)
                }
            } catch (e: Exception) {
                Logging.e("FilePicker", "Could not open the file picker", e)
            } finally {
                onDismiss()
            }
        }
    }
}

private fun tryOpenZenity(): List<Pair<String, ByteArray>>? {
    val process = try {
        ProcessBuilder("zenity", "--file-selection", "--multiple", "--separator=|", "--title=Select Files")
            .start()
    } catch (e: Exception) {
        return null
    }

    val output = process.inputStream.bufferedReader().use { it.readText() }.trim()
    process.waitFor()

    return if (process.exitValue() == 0 && output.isNotEmpty()) {
        output.split("|").map { path ->
            val file = File(path)
            file.name to Files.readAllBytes(file.toPath())
        }
    } else {
        emptyList()
    }
}

private fun openAwtFileDialog(): List<Pair<String, ByteArray>>? {
    var result: List<Pair<String, ByteArray>>? = null

    fun showDialog() {
        val dialog = FileDialog(null as Frame?, "Select Files", FileDialog.LOAD)
        dialog.isMultipleMode = true
        try {
            dialog.isVisible = true
            if (dialog.file == null) {
                result = emptyList()
                return
            }
            result = dialog.files.mapNotNull { file ->
                try {
                    file.name to Files.readAllBytes(file.toPath())
                } catch (e: Exception) {
                    Logging.e("FilePicker", "Could not read the selected file ${file.name}", e)
                    null
                }
            }
        } finally {
            dialog.dispose()
        }
    }

    try {
        if (SwingUtilities.isEventDispatchThread()) {
            showDialog()
        } else {
            SwingUtilities.invokeAndWait { showDialog() }
        }
    } catch (e: Exception) {
        Logging.e("FilePicker", "Could not open the file dialog", e)
        return null
    }
    return result
}
