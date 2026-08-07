package me.lampu.lampcord.shared.utils

import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalContext

@Composable
actual fun FilePicker(
    show: Boolean,
    onFileSelected: (List<Pair<String, ByteArray>>) -> Unit,
    onDismiss: () -> Unit
) {
    val context = LocalContext.current
    val launcher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetMultipleContents()
    ) { uris ->
        if (uris.isNotEmpty()) {
            val files = uris.mapNotNull { uri ->
                var name = "file"
                context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (nameIndex != -1 && cursor.moveToFirst()) {
                        name = cursor.getString(nameIndex)
                    }
                }
                
                context.contentResolver.openInputStream(uri)?.use { inputStream ->
                    name to inputStream.readBytes()
                }
            }
            onFileSelected(files)
        }
        onDismiss()
    }

    if (show) {
        LaunchedEffect(Unit) {
            launcher.launch("*/*")
        }
    }
}
