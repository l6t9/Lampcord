package me.lampu.lampcord.shared.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.awt.FileDialog
import java.awt.Frame
import java.io.File
import java.nio.file.Files

@Composable
actual fun FilePicker(
    show: Boolean,
    onFileSelected: (List<Pair<String, ByteArray>>) -> Unit,
    onDismiss: () -> Unit
) {
    if (show) {
        LaunchedEffect(Unit) {
            withContext(Dispatchers.IO) {
                val os = System.getProperty("os.name").lowercase()
                if (os.contains("linux")) {
                    val zenityFiles = tryOpenZenity()
                    if (zenityFiles != null) {
                        if (zenityFiles.isNotEmpty()) {
                            onFileSelected(zenityFiles)
                        }
                        onDismiss()
                        return@withContext
                    }
                }

                val fileDialog = FileDialog(null as Frame?, "Select Files", FileDialog.LOAD)
                fileDialog.isMultipleMode = true
                fileDialog.isVisible = true
                
                val selectedFiles = fileDialog.files
                if (selectedFiles != null && selectedFiles.isNotEmpty()) {
                    val files = selectedFiles.map { it.name to Files.readAllBytes(it.toPath()) }
                    onFileSelected(files)
                }
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
