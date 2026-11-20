package me.lampu.lampcord.shared.ui.components

import android.content.ContentResolver
import android.net.Uri
import android.provider.OpenableColumns
import android.webkit.MimeTypeMap
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.content.MediaType
import androidx.compose.foundation.content.ReceiveContentListener
import androidx.compose.foundation.content.consume
import androidx.compose.foundation.content.contentReceiver
import androidx.compose.foundation.content.hasMediaType
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.platform.LocalContext
import me.lampu.lampcord.shared.model.PendingFile
import me.lampu.lampcord.shared.utils.Logging

@OptIn(ExperimentalFoundationApi::class)
actual fun Modifier.keyboardImageReceiver(
    onMediaReceived: (PendingFile) -> Unit
): Modifier = composed {
    val context = LocalContext.current

    val listener = remember(onMediaReceived) {
        ReceiveContentListener { transferableContent ->
            Logging.d("Chatbox", "did u recieve?")
            // ignores if it isn't image
            if (!transferableContent.hasMediaType(MediaType.Image)) {
                return@ReceiveContentListener transferableContent
            }

            transferableContent.consume { item ->
                val uri = item.uri ?: return@consume false
                val contentResolver = context.contentResolver
                val mimeType = contentResolver.getType(uri)

                val bytes = contentResolver.openInputStream(uri)?.use { it.readBytes() }

                if (bytes != null) {
                    onMediaReceived(PendingFile(contentResolver.getFileName(uri, mimeType), bytes))
                    true
                } else {
                    false
                }
            }
        }
    }

    this.contentReceiver(listener)
}

private fun ContentResolver.getFileName(uri: Uri, mimeType: String?): String {
    // gets name via OpenableColumns
    query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
        if (cursor.moveToFirst()) {
            val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (nameIndex != -1) {
                val name = cursor.getString(nameIndex)
                if (!name.isNullOrBlank()) return name
            }
        }
    }

    // fallback: try to construct what it should be
    val fallbackName = uri.lastPathSegment ?: "keyboard_media_${System.currentTimeMillis()}"
    val extension = MimeTypeMap.getSingleton().getExtensionFromMimeType(mimeType)

    return if (extension != null && !fallbackName.endsWith(".$extension")) {
        "$fallbackName.$extension"
    } else {
        fallbackName
    }
}
